use std::path::PathBuf;

use super::{
    ast::AstKind,
    ast::AstNode,
    ast::ClassMember,
    expr_parser::ExprParser,
    expression::{Expression, ExpressionKind},
    interpreter::Program,
    lexer::Lexer,
    operator::Operator,
    token::{calculate_range, Token, TokenKind},
};
use pratt::PrattParser;
use std::ops::Range;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        expected: Vec<TokenKind>,
        actual: Option<Token>,
    },
    UnexpectedOperator(Token),
    Raw(String),
    Unknown(Range<usize>),
}

impl<'a> std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl<'a> From<String> for ParseError {
    fn from(value: String) -> Self {
        Self::Raw(value)
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
            pratt::PrattError::EmptyInput => Self::Raw("empty input".into()),
            _ => todo!("{:?}", value),
        }
    }
}

pub struct Parser<'a> {
    path: PathBuf,
    pub source: &'a str,
    pub offset: usize,
    pub lexer: itertools::MultiPeek<Lexer<'a>>,
    tokens: Vec<Token>,
    expr_parser: ExprParser<'a>,
}

impl<'a> Parser<'a> {
    pub fn new() -> Self {
        Self {
            path: "".into(),
            source: "",
            tokens: Vec::new(),
            offset: 0,
            lexer: itertools::multipeek(Lexer::new("", 0).into_iter()),
            expr_parser: ExprParser::default(),
        }
    }

    fn start_group(&mut self) {
        self.tokens.clear();
    }

    fn end_group(&mut self) -> Range<usize> {
        let res = self.get_group();
        self.tokens.clear();
        res
    }

    fn get_group(&self) -> Range<usize> {
        calculate_range(&self.tokens)
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
                | TokenKind::Export
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
                        if paren_depth == 0 && curly_depth == 0 {
                            match token.0 {
                                TokenKind::ClassIdentifier => {}
                                TokenKind::Arrow => {}
                                TokenKind::Hash => {}
                                _ => break,
                            }
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
            self.tokens.push(token.clone());
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
            .map(|kind| Expression::new(kind, self.get_group()))
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
                            args.push(Expression::new(
                                self.expr_parser
                                    .parse(&mut arg_tokens.clone().into_iter())?,
                                calculate_range(&arg_tokens),
                            ));
                        }
                        break;
                    }
                }
                TokenKind::Comma => {
                    if depth == 0 {
                        args.push(Expression::new(
                            self.expr_parser
                                .parse(&mut arg_tokens.clone().into_iter())?,
                            calculate_range(&arg_tokens),
                        ));
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

    fn advance(&mut self) -> Option<Token> {
        self.lexer.next().map(|t| {
            self.tokens.push(t.clone());
            t
        })
    }

    fn consume_either(&mut self, kinds: Vec<TokenKind>) -> Result<Token, ParseError> {
        if let Some(token) = self.advance() {
            if kinds.contains(&token.0) {
                Ok(token)
            } else if token.0 == TokenKind::NewLine {
                self.consume_either(kinds)
            } else {
                Err(ParseError::UnexpectedToken {
                    actual: Some(token),
                    expected: kinds,
                })
            }
        } else {
            Err(ParseError::UnexpectedToken {
                actual: None,
                expected: kinds,
            })
        }
    }

    fn consume(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        self.consume_either(vec![kind])
    }

    fn text(&self, token: &Token) -> &'a str {
        self.text_at(token.1.clone())
    }

    fn text_at(&self, range: Range<usize>) -> &'a str {
        let loc = range.start - self.offset..range.end - self.offset;
        &self.source[loc]
    }

    fn parse_fn_definition(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        self.consume(TokenKind::Fn)?;
        let name = self.consume(TokenKind::Identifier)?;
        self.consume(TokenKind::LParan)?;
        let args = self.parse_args()?;
        self.consume(TokenKind::LCurly)?;
        let body = self.parse_stmts()?;
        Ok(AstNode::new(
            AstKind::FunctionDefinition(
                self.text(&name).into(),
                args.iter().map(|a| a.to_string()).collect(),
                body,
            ),
            self.end_group(),
        ))
    }

    fn parse_static_fn_definition(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        self.consume(TokenKind::Static)?;
        self.consume(TokenKind::Fn)?;
        let name = self.consume(TokenKind::Identifier)?;
        self.consume(TokenKind::LParan)?;
        let args = self.parse_args()?;
        self.consume(TokenKind::LCurly)?;
        let body = self.parse_stmts()?;
        Ok(AstNode::new(
            AstKind::StaticFunctionDefinition(
                self.text(&name).into(),
                args.iter().map(|a| a.to_string()).collect(),
                body,
            ),
            self.end_group(),
        ))
    }

    fn parse_while_statement(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        let prev_token = self.consume(TokenKind::While)?;
        let cond = self.parse_expr(Some(prev_token))?;
        //TODO: skip whitespace before
        self.consume(TokenKind::LCurly)?;
        let block = self.parse_stmts()?;

        Ok(AstNode::new(
            AstKind::WhileStatement(cond, block),
            self.end_group(),
        ))
    }

    fn parse_if(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        let mut branches = Vec::new();

        let prev_token = self.consume(TokenKind::If)?;
        let cond = self.parse_expr(Some(prev_token))?;
        self.consume(TokenKind::LCurly)?;

        let block = self.parse_stmts()?;
        branches.push((cond, block));

        while let Some(token) = self.lexer.peek() {
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
                    let cond = Expression::new(ExpressionKind::BooleanLiteral("true".into()), 0..0);
                    self.consume(TokenKind::LCurly)?;
                    let block = self.parse_stmts()?;
                    branches.push((cond, block));
                }
                _ => break,
            }
        }

        self.lexer.reset_peek();

        Ok(AstNode::new(
            AstKind::IfStatement(branches),
            self.end_group(),
        ))
    }

    fn parse_var_assignment(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        let tok = self.consume(TokenKind::Identifier)?;
        let prev_token = self.consume(TokenKind::Equal)?;
        let value = self.parse_expr(Some(prev_token))?;

        Ok(AstNode::new(
            AstKind::VariableAssignment(self.text(&tok).into(), value),
            self.end_group(),
        ))
    }

    fn parse_plus_assignment(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        let ident = self.consume(TokenKind::Identifier)?;
        let prev_token = self.consume(TokenKind::PlusEqual)?;
        let value = self.parse_expr(Some(prev_token))?;

        Ok(AstNode::new(
            AstKind::PlusAssignment(self.text(&ident).into(), value),
            self.end_group(),
        ))
    }

    fn parse_minus_assignment(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        let ident = self.consume(TokenKind::Identifier)?;
        let prev_token = self.consume(TokenKind::MinusEqual)?;
        let value = self.parse_expr(Some(prev_token))?;

        Ok(AstNode::new(
            AstKind::MinusAssignment(self.text(&ident).into(), value),
            self.end_group(),
        ))
    }

    fn parse_times_assignment(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        let ident = self.consume(TokenKind::Identifier)?;
        let prev_token = self.consume(TokenKind::StarEqual)?;
        let value = self.parse_expr(Some(prev_token))?;

        Ok(AstNode::new(
            AstKind::TimesAssignment(self.text(&ident).into(), value),
            self.end_group(),
        ))
    }

    fn parse_divide_assignment(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        let ident = self.consume(TokenKind::Identifier)?;
        let prev_token = self.consume(TokenKind::SlashEqual)?;
        let value = self.parse_expr(Some(prev_token))?;

        Ok(AstNode::new(
            AstKind::DivideAssignment(self.text(&ident).into(), value),
            self.end_group(),
        ))
    }

    fn parse_class_definition(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        self.consume(TokenKind::Class)?;
        let name = self.consume(TokenKind::ClassIdentifier)?;
        self.consume(TokenKind::LCurly)?;
        let body = self.parse_stmts()?;
        let members = body
            .iter()
            .map(|ast| match &ast.kind {
                AstKind::VariableDefinition(a, b) => ClassMember::Field(a.clone(), b.clone()),
                AstKind::FunctionDefinition(a, b, c) => {
                    ClassMember::Function(a.clone(), b.clone(), c.clone())
                }
                AstKind::StaticFunctionDefinition(a, b, c) => {
                    ClassMember::StaticFunction(a.clone(), b.clone(), c.clone())
                }
                AstKind::OperatorImplementation(a, b, c) => {
                    ClassMember::Operator(a.clone(), b.clone(), c.clone())
                }
                _ => panic!("not allowed"),
            })
            .collect::<Vec<ClassMember>>();

        Ok(AstNode::new(
            AstKind::ClassDefinition(self.text(&name).into(), members),
            todo!(),
        ))
    }

    fn parse_op_implementation(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        self.consume(TokenKind::Op)?;
        let t = self.consume(TokenKind::Identifier)?;
        let op = match self.text(&t) {
            "add" => Operator::Add,
            "dot" => Operator::Dot,
            text => panic!("Unknown operator function {}", text),
        };
        self.consume(TokenKind::LParan)?;
        let args = self.parse_args()?;
        self.consume(TokenKind::LCurly)?;
        let body = self.parse_stmts()?;

        Ok(AstNode::new(
            AstKind::OperatorImplementation(op, args.iter().map(|a| a.to_string()).collect(), body),
            self.end_group(),
        ))
    }

    fn parse_import_statement(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        self.consume(TokenKind::Import)?;
        let mut parts = Vec::new();

        while let Some(token) = self.lexer.peek() {
            match token.0 {
                TokenKind::Dot | TokenKind::Identifier => {
                    let token = self.lexer.next().unwrap();
                    self.tokens.push(token.clone());
                    parts.push(self.text(&token));
                }
                _ => break,
            }
        }

        let tokens = self.end_group();

        Ok(AstNode::new(
            AstKind::ImportStatement(parts.join("")),
            tokens,
        ))
    }

    fn parse_export_statement(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        self.consume(TokenKind::Export)?;

        if let Some(token) = self.lexer.peek().cloned() {
            let ast = match token.0 {
                TokenKind::ClassIdentifier => AstNode::new(
                    AstKind::Expression(Expression::new(
                        ExpressionKind::ClassIdentifier(self.text(&token).to_string()),
                        token.1.clone(),
                    )),
                    token.1.clone(),
                ),
                TokenKind::Var => self.parse_var_definition()?,
                TokenKind::Class => self.parse_class_definition()?,
                TokenKind::Fn => self.parse_fn_definition()?,
                TokenKind::Identifier => AstNode::new(
                    AstKind::Expression(Expression::new(
                        ExpressionKind::Identifier(self.text(&token).to_string()),
                        token.1.clone(),
                    )),
                    token.1.clone(),
                ),
                _ => panic!("Expected either a class, variable or function"),
            };

            Ok(AstNode::new(
                AstKind::ExportStatement(Box::new(ast)),
                self.end_group(),
            ))
        } else {
            panic!("expected either a class or a variable");
        }
    }

    fn parse_return_statement(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        let prev_token = self.consume(TokenKind::Return)?;

        let value = if let Some(token) = self.lexer.peek() {
            match token.0 {
                TokenKind::SemiColon => Expression::new(ExpressionKind::Null, token.1.clone()),
                _ => {
                    self.lexer.reset_peek();
                    self.parse_expr(Some(prev_token))?
                }
            }
        } else {
            todo!()
        };

        Ok(AstNode::new(
            AstKind::ReturnStatement(value),
            self.end_group(),
        ))
    }

    fn parse_comment(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        let mut lines = Vec::new();

        self.start_group();

        let token = self.consume(TokenKind::Comment).unwrap();

        let mut line = (token.1.end, 0);

        while let Some(token) = self.lexer.next() {
            match token.0 {
                TokenKind::NewLine => {
                    lines.push(self.text_at(line.0..line.1).to_string());
                    if let Some(TokenKind::Comment) = self.lexer.peek().map(|x| x.0.clone()) {
                        let token = self.lexer.next().unwrap();
                        line = (token.1.end, 0);
                    } else {
                        self.lexer.reset_peek();
                        break;
                    }
                }
                _ => {
                    if token.1.end > line.1 {
                        line.1 = token.1.end;
                    }
                }
            }
        }

        Ok(AstNode::new(AstKind::Comment(lines), self.end_group()))
    }

    fn parse_documentation(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        let mut lines = Vec::new();

        let token = self.consume(TokenKind::TripleSlash).unwrap();

        let mut line = (token.1.end, 0);

        while let Some(token) = self.lexer.next() {
            match token.0 {
                TokenKind::NewLine => {
                    lines.push(self.source[line.0..line.1].to_string());
                    if let Some(TokenKind::TripleSlash) = self.lexer.peek().map(|x| x.0.clone()) {
                        let token = self.lexer.next().unwrap();
                        line = (token.1.end, 0);
                    } else {
                        self.lexer.reset_peek();
                        break;
                    }
                }
                _ => {
                    if token.1.end > line.1 {
                        line.1 = token.1.end;
                    }
                }
            }
        }

        Ok(AstNode::new(
            AstKind::Documentation(lines),
            self.end_group(),
        ))
    }

    fn parse_var_definition(&mut self) -> Result<AstNode, ParseError> {
        self.start_group();
        self.consume(TokenKind::Var)?;
        let tok = self.consume_either(vec![TokenKind::Identifier, TokenKind::LBracket])?;
        match &tok.0 {
            TokenKind::Identifier => {
                let token = self.consume_either(vec![TokenKind::SemiColon, TokenKind::Equal])?;
                let value = match token.0 {
                    TokenKind::SemiColon => Expression::new(ExpressionKind::Null, token.1.clone()),
                    TokenKind::Equal => self.parse_expr(None)?,
                    _ => unreachable!(),
                };

                Ok(AstNode::new(
                    AstKind::VariableDefinition(self.text(&tok).into(), value),
                    self.end_group(),
                ))
            }
            TokenKind::LBracket => {
                let identifiers = self
                    .parse_args()?
                    .iter()
                    .map(|t| match &t.kind {
                        ExpressionKind::Identifier(x) => x.clone(),
                        _ => panic!(),
                    })
                    .collect::<Vec<_>>();
                self.consume(TokenKind::Equal)?;
                let value = self.parse_expr(None)?;
                Ok(AstNode::new(
                    AstKind::ArrayVariableDefinition(identifiers, value),
                    self.end_group(),
                ))
            }
            _ => unreachable!(),
        }
    }

    fn parse_stmts(&mut self) -> Result<Vec<AstNode>, ParseError> {
        let mut stmts = Vec::new();
        let mut depth = 0;

        while let Some(token) = self.lexer.peek() {
            let ast = match token.0 {
                TokenKind::Comment => self.parse_comment(),
                TokenKind::TripleSlash => self.parse_documentation(),
                TokenKind::Return => self.parse_return_statement(),
                TokenKind::Break => Ok(AstNode::new(AstKind::BreakStatement, token.1.clone())),
                TokenKind::Continue => {
                    Ok(AstNode::new(AstKind::ContinueStatement, token.1.clone()))
                }
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
                    let loc = token.1.clone();
                    if let Some(next) = self.lexer.peek() {
                        match next.0 {
                            TokenKind::LCurly => {
                                drop(next);
                                self.lexer.reset_peek();
                                Ok(AstNode::new(
                                    AstKind::Expression(self.parse_expr(None)?),
                                    loc,
                                ))
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
                                let loc = token.1.clone();
                                Ok(AstNode::new(
                                    AstKind::Expression(self.parse_expr(None)?),
                                    loc,
                                ))
                            }
                            _ => {
                                self.start_group();
                                self.parse_expr(None)
                                    .map(|x| AstNode::new(AstKind::Expression(x), self.end_group()))
                            }
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
        let mut lexer = Lexer::new(self.source, self.offset);
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
        self.expr_parser.offset = offset;
        self.offset = offset;
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
                ParseError::Raw(msg) => Err(msg),
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
