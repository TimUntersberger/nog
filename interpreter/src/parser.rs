use std::path::PathBuf;

use super::{
    ast::Ast,
    ast::ClassMember,
    expr_parser::ExprParser,
    expression::Expression,
    interpreter::Program,
    lexer::Lexer,
    operator::Operator,
    token::{Token, TokenKind},
};
use pratt::PrattParser;
use std::ops::Range;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        expected: TokenKind,
        actual: Option<Token>,
    },
    UnexpectedOperator(Token),
    Unknown(Range<usize>),
}

impl<'a> std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl<'a> From<pratt::PrattError<Token, ParseError>> for ParseError {
    fn from(value: pratt::PrattError<Token, ParseError>) -> Self {
        match value {
            pratt::PrattError::UserError(e) => e,
            pratt::PrattError::UnexpectedInfix(i) => Self::UnexpectedOperator(i),
            pratt::PrattError::UnexpectedNilfix(_) => todo!(),
            pratt::PrattError::UnexpectedPrefix(i) => Self::UnexpectedOperator(i),
            pratt::PrattError::UnexpectedPostfix(i) => Self::UnexpectedOperator(i),
            _ => todo!(),
        }
    }
}

pub struct Parser<'a> {
    path: PathBuf,
    pub source: &'a str,
    pub lexer: itertools::MultiPeek<Lexer<'a>>,
    expr_parser: ExprParser<'a>,
}

impl<'a> Parser<'a> {
    pub fn new() -> Self {
        Self {
            path: "".into(),
            source: "",
            lexer: itertools::multipeek(Lexer::new("", 0).into_iter()),
            expr_parser: ExprParser::default(),
        }
    }

    fn parse_expr(&mut self, prev_token: Option<Token>) -> Result<Expression, ParseError> {
        let mut tokens = Vec::new();
        let mut paren_depth = 0;
        let mut curly_depth = 0;
        let mut previous_token: Option<Token> = prev_token.clone();

        self.lexer.reset_peek();

        while let Some(token) = self.lexer.peek().cloned() {
            match token.0 {
                TokenKind::Fn
                | TokenKind::Var
                | TokenKind::Class
                | TokenKind::While
                | TokenKind::Import
                | TokenKind::Return
                | TokenKind::ElseIf
                | TokenKind::If => {
                    if !tokens.is_empty() && paren_depth == 0 && curly_depth == 0 {
                        break;
                    }
                }
                TokenKind::Comment => {
                    if !tokens.is_empty() && paren_depth == 0 && curly_depth == 0 {
                        break;
                    }
                }
                TokenKind::NewLine => {
                    self.lexer.next();
                    continue;
                }
                TokenKind::SemiColon => {
                    if paren_depth == 0 && curly_depth == 0 {
                        break;
                    }
                }
                TokenKind::LParan => paren_depth += 1,
                TokenKind::RParan => {
                    paren_depth -= 1;
                    if paren_depth == -1 {
                        break;
                    }
                }
                TokenKind::Identifier => {
                    if let Some(prev_token) = previous_token {
                        if paren_depth == 0 && curly_depth == 0 {
                            match prev_token.0 {
                                TokenKind::RParan
                                | TokenKind::RCurly
                                | TokenKind::Identifier
                                | TokenKind::StringLiteral
                                | TokenKind::NumberLiteral
                                | TokenKind::BooleanLiteral
                                | TokenKind::PlusPlus
                                | TokenKind::MinusMinus
                                | TokenKind::ClassIdentifier
                                | TokenKind::RBracket => break,
                                _ => {}
                            }
                        }
                    }
                }
                TokenKind::LCurly => {
                    if let Some(token) = previous_token {
                        match token.0 {
                            TokenKind::ClassIdentifier => {}
                            TokenKind::Arrow => {}
                            TokenKind::Hash => {}
                            _ => break,
                        }
                    }
                    curly_depth += 1
                }
                TokenKind::RCurly => {
                    curly_depth -= 1;
                    if curly_depth == -1 {
                        break;
                    }
                }
                _ => {}
            }

            let token = self.lexer.next().unwrap();
            previous_token = Some(token.clone());
            tokens.push(token);
        }

        self.lexer.reset_peek();

        if let Some(token) = prev_token {
            self.expr_parser.prev_token = token;
        } else {
            self.expr_parser.prev_token = Token::default();
        }

        self.expr_parser
            .parse(&mut tokens.into_iter())
            .map_err(|e| e.into())
    }

    fn parse_args(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut depth = 0;
        let mut args = Vec::new();
        let mut arg_tokens = Vec::new();

        while let Some(next) = self.lexer.next() {
            match next.0 {
                TokenKind::RBracket | TokenKind::RParan | TokenKind::RCurly => {
                    if depth != 0 {
                        depth -= 1;
                    } else {
                        if !arg_tokens.is_empty() {
                            args.push(
                                self.expr_parser
                                    .parse(&mut arg_tokens.clone().into_iter())?,
                            );
                        }
                        break;
                    }
                }
                TokenKind::Comma => {
                    if depth == 0 {
                        args.push(
                            self.expr_parser
                                .parse(&mut arg_tokens.clone().into_iter())?,
                        );
                        arg_tokens.clear();
                        continue;
                    }
                }
                TokenKind::LParan | TokenKind::LBracket | TokenKind::LCurly => depth += 1,
                _ => {}
            }
            arg_tokens.push(next);
        }

        Ok(args)
    }

    fn consume(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        if let Some(token) = self.lexer.next() {
            if token.0 == kind {
                Ok(token)
            } else if token.0 == TokenKind::NewLine {
                self.consume(kind)
            } else {
                Err(ParseError::UnexpectedToken {
                    actual: Some(token),
                    expected: kind,
                })
            }
        } else {
            Err(ParseError::UnexpectedToken {
                actual: None,
                expected: kind,
            })
        }
    }

    fn text(&self, token: &Token) -> &'a str {
        &self.source[token.1.clone()]
    }

    fn parse_fn_definition(&mut self) -> Result<Ast, ParseError> {
        self.consume(TokenKind::Fn)?;
        let name = self.consume(TokenKind::Identifier)?;
        self.consume(TokenKind::LParan)?;
        let args = self.parse_args()?;
        self.consume(TokenKind::LCurly)?;
        let body = self.parse_stmts()?;
        Ok(Ast::FunctionDefinition(
            self.text(&name).into(),
            args.iter().map(|a| a.to_string()).collect(),
            body,
        ))
    }

    fn parse_static_fn_definition(&mut self) -> Result<Ast, ParseError> {
        self.consume(TokenKind::Static)?;
        self.consume(TokenKind::Fn)?;
        let name = self.consume(TokenKind::Identifier)?;
        self.consume(TokenKind::LParan)?;
        let args = self.parse_args()?;
        self.consume(TokenKind::LCurly)?;
        let body = self.parse_stmts()?;
        Ok(Ast::StaticFunctionDefinition(
            self.text(&name).into(),
            args.iter().map(|a| a.to_string()).collect(),
            body,
        ))
    }

    fn parse_while_statement(&mut self) -> Result<Ast, ParseError> {
        let prev_token = self.consume(TokenKind::While)?;
        let cond = self.parse_expr(Some(prev_token))?;
        //TODO: skip whitespace before
        self.consume(TokenKind::LCurly)?;
        let block = self.parse_stmts()?;

        Ok(Ast::WhileStatement(cond, block))
    }

    fn parse_if(&mut self) -> Result<Ast, ParseError> {
        let mut branches = Vec::new();

        let prev_token = self.consume(TokenKind::If)?;
        let cond = self.parse_expr(Some(prev_token))?;
        self.consume(TokenKind::LCurly)?;

        let block = self.parse_stmts()?;
        branches.push((cond, block));

        while let Some(token) = dbg!(self.lexer.peek()) {
            match token.0 {
                TokenKind::NewLine => {
                    self.lexer.next();
                }
                TokenKind::ElseIf => {
                    let prev_token = self.consume(TokenKind::ElseIf)?;
                    let cond = self.parse_expr(Some(prev_token))?;
                    self.consume(TokenKind::LCurly)?;
                    let block = self.parse_stmts()?;
                    branches.push((cond, block));
                }
                TokenKind::Else => {
                    self.consume(TokenKind::Else)?;
                    let cond = Expression::BooleanLiteral("true".into());
                    self.consume(TokenKind::LCurly)?;
                    let block = self.parse_stmts()?;
                    branches.push((cond, block));
                }
                _ => break,
            }
        }

        self.lexer.reset_peek();

        Ok(Ast::IfStatement(branches))
    }

    fn parse_var_assignment(&mut self) -> Result<Ast, ParseError> {
        let ident = self.consume(TokenKind::Identifier)?;
        let prev_token = self.consume(TokenKind::Equal)?;
        let value = self.parse_expr(Some(prev_token))?;

        Ok(Ast::VariableAssignment(self.text(&ident).into(), value))
    }

    fn parse_plus_assignment(&mut self) -> Result<Ast, ParseError> {
        let ident = self.consume(TokenKind::Identifier)?;
        let prev_token = self.consume(TokenKind::PlusEqual)?;
        let value = self.parse_expr(Some(prev_token))?;

        Ok(Ast::PlusAssignment(self.text(&ident).into(), value))
    }

    fn parse_minus_assignment(&mut self) -> Result<Ast, ParseError> {
        let ident = self.consume(TokenKind::Identifier)?;
        let prev_token = self.consume(TokenKind::MinusEqual)?;
        let value = self.parse_expr(Some(prev_token))?;

        Ok(Ast::MinusAssignment(self.text(&ident).into(), value))
    }

    fn parse_times_assignment(&mut self) -> Result<Ast, ParseError> {
        let ident = self.consume(TokenKind::Identifier)?;
        let prev_token = self.consume(TokenKind::StarEqual)?;
        let value = self.parse_expr(Some(prev_token))?;

        Ok(Ast::TimesAssignment(self.text(&ident).into(), value))
    }

    fn parse_divide_assignment(&mut self) -> Result<Ast, ParseError> {
        let ident = self.consume(TokenKind::Identifier)?;
        let prev_token = self.consume(TokenKind::SlashEqual)?;
        let value = self.parse_expr(Some(prev_token))?;

        Ok(Ast::DivideAssignment(self.text(&ident).into(), value))
    }

    fn parse_class_definition(&mut self) -> Result<Ast, ParseError> {
        self.consume(TokenKind::Class)?;
        let name = self.consume(TokenKind::ClassIdentifier)?;
        self.consume(TokenKind::LCurly)?;
        let body = self.parse_stmts()?;
        let members = body
            .iter()
            .map(|ast| match ast {
                Ast::VariableDefinition(a, b) => ClassMember::Field(a.clone(), b.clone()),
                Ast::FunctionDefinition(a, b, c) => {
                    ClassMember::Function(a.clone(), b.clone(), c.clone())
                }
                Ast::StaticFunctionDefinition(a, b, c) => {
                    ClassMember::StaticFunction(a.clone(), b.clone(), c.clone())
                }
                Ast::OperatorImplementation(a, b, c) => {
                    ClassMember::Operator(a.clone(), b.clone(), c.clone())
                }
                _ => panic!("not allowed"),
            })
            .collect::<Vec<ClassMember>>();

        Ok(Ast::ClassDefinition(self.text(&name).into(), members))
    }

    fn parse_op_implementation(&mut self) -> Result<Ast, ParseError> {
        self.consume(TokenKind::Op)?;
        let op = self
            .lexer
            .next()
            .map(|t| self.text(&t))
            .map(|text| match text {
                "add" => Operator::Add,
                "dot" => Operator::Dot,
                _ => panic!("Unknown operator function {}", text),
            })
            .unwrap();
        self.consume(TokenKind::LParan)?;
        let args = self.parse_args()?;
        self.consume(TokenKind::LCurly)?;
        let body = self.parse_stmts()?;

        Ok(Ast::OperatorImplementation(
            op,
            args.iter().map(|a| a.to_string()).collect(),
            body,
        ))
    }

    fn parse_import_statement(&mut self) -> Result<Ast, ParseError> {
        self.consume(TokenKind::Import)?;
        let mut tokens = Vec::new();

        while let Some(token) = self.lexer.peek() {
            match token.0 {
                TokenKind::Dot | TokenKind::Identifier => {
                    let token = self.lexer.next().unwrap();
                    tokens.push(self.text(&token))
                }
                _ => break,
            }
        }

        Ok(Ast::ImportStatement(tokens.join("")))
    }

    fn parse_export_statement(&mut self) -> Result<Ast, ParseError> {
        self.consume(TokenKind::Export)?;

        if let Some(token) = self.lexer.peek().cloned() {
            let ast = match token.0 {
                TokenKind::ClassIdentifier => {
                    Ast::Expression(Expression::ClassIdentifier(self.text(&token).to_string()))
                }
                TokenKind::Var => self.parse_var_definition()?,
                TokenKind::Class => self.parse_class_definition()?,
                TokenKind::Fn => self.parse_fn_definition()?,
                TokenKind::Identifier => {
                    Ast::Expression(Expression::Identifier(self.text(&token).to_string()))
                }
                _ => panic!("Expected either a class, variable or function"),
            };

            Ok(Ast::ExportStatement(Box::new(ast)))
        } else {
            panic!("expected either a class or a variable");
        }
    }

    fn parse_return_statement(&mut self) -> Result<Ast, ParseError> {
        let prev_token = self.consume(TokenKind::Return)?;

        let value = if let Some(token) = self.lexer.peek() {
            match token.0 {
                TokenKind::SemiColon => Expression::Null,
                _ => {
                    self.lexer.reset_peek();
                    self.parse_expr(Some(prev_token))?
                }
            }
        } else {
            todo!()
        };

        Ok(Ast::ReturnStatement(value))
    }

    fn parse_documentation(&mut self) -> Result<Ast, ParseError> {
        let mut lines = Vec::new();
        let mut line = (0, 0);

        self.consume(TokenKind::TripleSlash).unwrap();

        while let Some(token) = self.lexer.next() {
            match token.0 {
                TokenKind::NewLine => {
                    lines.push(self.source[line.0..line.1].to_string());
                    line = (0, 0);
                    if let Some(TokenKind::TripleSlash) = self.lexer.peek().map(|x| x.0.clone()) {
                        self.lexer.next();
                    } else {
                        self.lexer.reset_peek();
                        break;
                    }
                }
                _ => {
                    if line.0 == 0 {
                        line.0 = token.1.start;
                    }

                    if token.1.end > line.1 {
                        line.1 = token.1.end;
                    }
                },
            }
        }

        Ok(Ast::Documentation(lines))
    }

    fn parse_var_definition(&mut self) -> Result<Ast, ParseError> {
        self.consume(TokenKind::Var)?;
        let ident = self.consume(TokenKind::Identifier)?;

        let value = if let Some(token) = self.lexer.peek() {
            match token.0 {
                TokenKind::SemiColon => Expression::Null,
                _ => {
                    self.consume(TokenKind::Equal)?;
                    self.parse_expr(None)?
                }
            }
        } else {
            todo!()
        };

        Ok(Ast::VariableDefinition(self.text(&ident).into(), value))
    }

    fn parse_stmts(&mut self) -> Result<Vec<Ast>, ParseError> {
        let mut stmts = Vec::new();
        let mut depth = 0;

        while let Some(token) = self.lexer.peek() {
            let ast = match token.0 {
                TokenKind::Comment => {
                    while let Some(token) = self.lexer.next() {
                        match token.0 {
                            TokenKind::NewLine => break,
                            _ => {}
                        }
                    }
                    continue;
                }
                TokenKind::TripleSlash => self.parse_documentation(),
                TokenKind::Return => self.parse_return_statement(),
                TokenKind::Break => Ok(Ast::BreakStatement),
                TokenKind::Continue => Ok(Ast::ContinueStatement),
                TokenKind::While => self.parse_while_statement(),
                TokenKind::Class => self.parse_class_definition(),
                TokenKind::Var => self.parse_var_definition(),
                TokenKind::Op => self.parse_op_implementation(),
                TokenKind::Fn => self.parse_fn_definition(),
                TokenKind::Static => self.parse_static_fn_definition(),
                TokenKind::Import => self.parse_import_statement(),
                TokenKind::Export => self.parse_export_statement(),
                TokenKind::If => self.parse_if(),
                TokenKind::ClassIdentifier => {
                    if let Some(token) = self.lexer.peek() {
                        match token.0 {
                            TokenKind::LCurly => {
                                drop(token);
                                self.lexer.reset_peek();
                                Ok(Ast::Expression(self.parse_expr(None)?))
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        unreachable!()
                    }
                }
                TokenKind::Identifier => {
                    if let Some(token) = self.lexer.peek() {
                        match token.0 {
                            TokenKind::Equal => self.parse_var_assignment(),
                            TokenKind::PlusEqual => self.parse_plus_assignment(),
                            TokenKind::MinusEqual => self.parse_minus_assignment(),
                            TokenKind::StarEqual => self.parse_times_assignment(),
                            TokenKind::SlashEqual => self.parse_divide_assignment(),
                            TokenKind::Dot | TokenKind::DoubleColon => {
                                Ok(Ast::Expression(self.parse_expr(None)?))
                            }
                            _ => self.parse_expr(None).map(|x| Ast::Expression(x)),
                        }
                    } else {
                        unreachable!()
                    }
                }
                TokenKind::RCurly => {
                    self.lexer.next();
                    if depth == 0 {
                        break;
                    } else {
                        depth -= 1;
                        continue;
                    }
                }
                _ => {
                    self.lexer.next();
                    continue;
                }
            }?;

            stmts.push(ast);
        }

        Ok(stmts)
    }

    fn position(&self, location: &std::ops::Range<usize>) -> Option<(usize, usize)> {
        let mut lexer = Lexer::new(self.source, 0);
        let mut line = 1;
        let mut col_start = 1;

        while let Some(token) = lexer.next() {
            if token.1 == *location {
                let col = location.start - col_start;
                return Some((line, col));
            }
            match token.0 {
                TokenKind::NewLine => {
                    line += 1;
                    col_start = token.1.start;
                }
                _ => {}
            }
        }

        None
    }

    pub fn set_source(&mut self, path: PathBuf, source: &'a str, offset: usize) {
        self.path = path;
        self.source = source;
        self.expr_parser.source = source;
        self.lexer = itertools::multipeek(Lexer::new(source, offset));
    }

    pub fn parse(&'a mut self) -> Result<Program, String> {
        match self.parse_stmts() {
            Ok(stmts) => Ok(Program {
                path: self.path.clone(),
                source: self.source,
                stmts,
            }),
            Err(e) => match e {
                ParseError::UnexpectedOperator(token) => {
                    let (line, col) = self.position(&token.1).unwrap();
                    Err(format!(
                        "Encountered unexpected operator '{}' at {},{}",
                        self.text(&token),
                        line,
                        col
                    ))
                }
                ParseError::Unknown(loc) => {
                    let (line, col) = self.position(&loc).unwrap();
                    Err(format!("Encountered an unknown error at {},{}", line, col))
                }
                ParseError::UnexpectedToken { expected, actual } => {
                    if let Some(actual) = actual {
                        let (line, col) = self.position(&actual.1).unwrap();
                        Err(format!(
                            "Expected {:?}, but found {:?} at {},{}",
                            expected,
                            self.text(&actual),
                            line,
                            col
                        ))
                    } else {
                        Err(format!("Expected {:?}, but found EOF", expected))
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::Parser;
    use crate::ast::Ast;
    use crate::ast::Ast::*;
    use crate::expression::Expression;
    use crate::lexer::Lexer;
    use crate::operator::Operator;

    fn expect(code: &str, ast: Ast) {
        let mut parser = Parser::new();
        parser.expr_parser.source = code;
        parser.source = code;
        parser.lexer = itertools::multipeek(Lexer::new(code, 0));
        let prog = parser.parse().unwrap();
        assert_eq!(prog.stmts, vec![ast]);
    }

    #[test]
    pub fn if_stmt() {
        expect(r#"if true {}"#, IfStatement(vec![(true.into(), vec![])]));
    }

    #[test]
    pub fn if_stmt_with_else() {
        expect(
            r#"if true {} else {}"#,
            IfStatement(vec![(true.into(), vec![]), (true.into(), vec![])]),
        );
    }

    #[test]
    pub fn plus_shortcut() {
        expect(r#"test += 1"#, PlusAssignment("test".into(), 1.into()));
    }

    #[test]
    pub fn minus_shortcut() {
        expect(r#"test -= 1"#, MinusAssignment("test".into(), 1.into()));
    }

    #[test]
    pub fn times_shortcut() {
        expect(r#"test *= 1"#, TimesAssignment("test".into(), 1.into()));
    }

    #[test]
    pub fn divide_shortcut() {
        expect(r#"test /= 1"#, DivideAssignment("test".into(), 1.into()));
    }

    #[test]
    pub fn while_loop_with_if_stmt() {
        expect(
            r#"
                while true {
                    if true {}
                    print();
                }
            "#,
            WhileStatement(
                true.into(),
                vec![
                    IfStatement(vec![(true.into(), vec![])]),
                    Expression(Expression::PostOp(
                        Box::new(Expression::Identifier("print".into())),
                        Operator::Call,
                        Some(Box::new(Expression::ArrayLiteral(vec![]))),
                    )),
                ],
            ),
        );
    }
}
