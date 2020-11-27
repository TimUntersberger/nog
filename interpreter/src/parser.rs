use std::path::PathBuf;

use super::{
    ast::Ast, ast::ClassMember, expr_parser::ExprParser, expression::Expression,
    interpreter::Program, lexer::Lexer, operator::Operator, token::Token,
};
use pratt::PrattParser;

pub struct Parser<'a> {
    path: PathBuf,
    pub lexer: itertools::MultiPeek<Lexer<'a>>,
    expr_parser: ExprParser<'a>,
}

impl<'a> Parser<'a> {
    pub fn new() -> Self {
        Self {
            path: "".into(),
            lexer: itertools::multipeek(Lexer::new("").into_iter()),
            expr_parser: ExprParser::default(),
        }
    }

    fn parse_expr(&mut self, prev_token: Option<Token<'a>>) -> Result<Expression, String> {
        let mut tokens = Vec::new();
        let mut paren_depth = 0;
        let mut curly_depth = 0;
        let mut previous_token: Option<Token> = None;

        self.lexer.reset_peek();

        while let Some(token) = self.lexer.peek() {
            match token {
                Token::NewLine(_) => {
                    self.lexer.next();
                    continue;
                }
                Token::SemiColon(_) => {
                    if paren_depth == 0 && curly_depth == 0 {
                        break;
                    }
                }
                Token::LParan(_) => paren_depth += 1,
                Token::RParan(_) => {
                    paren_depth -= 1;
                    if paren_depth == -1 {
                        break;
                    }
                }
                Token::LCurly(_) => {
                    match previous_token {
                        Some(Token::ClassIdentifier(_)) => {}
                        Some(Token::Arrow(_)) => {}
                        Some(Token::Hash(_)) => {}
                        _ => break,
                    }
                    curly_depth += 1
                }
                Token::RCurly(_) => {
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
        }

        self.expr_parser
            .parse(&mut tokens.into_iter())
            .map_err(|e| format!("{:?}", e))
    }

    fn parse_args(&mut self) -> Result<Vec<Expression>, String> {
        let mut depth = 0;
        let mut args = Vec::new();
        let mut arg_tokens = Vec::new();

        while let Some(next) = self.lexer.next() {
            match next {
                Token::RBracket(_) | Token::RParan(_) | Token::RCurly(_) => {
                    if depth != 0 {
                        depth -= 1;
                    } else {
                        if !arg_tokens.is_empty() {
                            args.push(
                                self.expr_parser
                                    .parse(&mut arg_tokens.clone().into_iter())
                                    .map_err(|e| format!("{:?}", e))?,
                            );
                        }
                        break;
                    }
                }
                Token::Comma(_) => {
                    if depth == 0 {
                        args.push(
                            self.expr_parser
                                .parse(&mut arg_tokens.clone().into_iter())
                                .map_err(|e| format!("{:?}", e))?,
                        );
                        arg_tokens.clear();
                        continue;
                    }
                }
                Token::LParan(_) | Token::LBracket(_) | Token::LCurly(_) => depth += 1,
                _ => {}
            }
            arg_tokens.push(next);
        }

        Ok(args)
    }

    fn parse_fn_definition(&mut self) -> Result<Ast, String> {
        consume!(self.lexer, Token::Fn, "fn")?;
        let name = consume!(self.lexer, Token::Identifier, "identifier")?;
        consume!(self.lexer, Token::LParan, "(")?;
        let args = self.parse_args()?;
        consume!(self.lexer, Token::LCurly, "{")?;
        let body = self.parse_stmts()?;
        Ok(Ast::FunctionDefinition(
            name.text().to_string(),
            args.iter().map(|a| a.to_string()).collect(),
            body,
        ))
    }

    fn parse_static_fn_definition(&mut self) -> Result<Ast, String> {
        consume!(self.lexer, Token::Static, "static")?;
        consume!(self.lexer, Token::Fn, "fn")?;
        let name = consume!(self.lexer, Token::Identifier, "identifier")?;
        consume!(self.lexer, Token::LParan, "(")?;
        let args = self.parse_args()?;
        consume!(self.lexer, Token::LCurly, "{")?;
        let body = self.parse_stmts()?;
        Ok(Ast::StaticFunctionDefinition(
            name.text().to_string(),
            args.iter().map(|a| a.to_string()).collect(),
            body,
        ))
    }

    fn parse_while_statement(&mut self) -> Result<Ast, String> {
        let prev_token = consume!(self.lexer, Token::While, "while")?;
        let cond = self.parse_expr(Some(prev_token))?;
        consume!(self.lexer, Token::LCurly, "{", true)?;
        let block = self.parse_stmts()?;

        Ok(Ast::WhileStatement(cond, block))
    }

    fn parse_if(&mut self) -> Result<Ast, String> {
        let mut branches = Vec::new();

        let prev_token = consume!(self.lexer, Token::If, "if")?;
        let cond = self.parse_expr(Some(prev_token))?;
        consume!(self.lexer, Token::LCurly, "{", true)?;
        let block = self.parse_stmts()?;
        branches.push((cond, block));

        while let Some(token) = &self.lexer.peek() {
            match token {
                Token::ElseIf(_) => {
                    self.lexer.next();
                    let prev_token = consume!(self.lexer, Token::ElseIf, "elif")?;
                    let cond = self.parse_expr(Some(prev_token))?;
                    consume!(self.lexer, Token::LCurly, "{", true)?;
                    let block = self.parse_stmts()?;
                    branches.push((cond, block));
                }
                Token::Else(_) => {
                    self.lexer.next();
                    consume!(self.lexer, Token::Else, "else")?;
                    let cond = Expression::BooleanLiteral(true);
                    consume!(self.lexer, Token::LCurly, "{", true)?;
                    let block = self.parse_stmts()?;
                    branches.push((cond, block));
                }
                _ => break,
            }
        }

        Ok(Ast::IfStatement(branches))
    }

    fn parse_var_assignment(&mut self) -> Result<Ast, String> {
        let ident = match self.lexer.next() {
            Some(Token::Identifier(data)) => data.value.to_string(),
            _ => todo!(),
        };
        let prev_token = consume!(self.lexer, Token::Equal, "=")?;
        let value = self.parse_expr(Some(prev_token))?;

        Ok(Ast::VariableAssignment(ident, value))
    }

    fn parse_class_definition(&mut self) -> Result<Ast, String> {
        consume!(self.lexer, Token::Class, "class")?;
        let name = consume!(self.lexer, Token::ClassIdentifier, "class identifier")?;
        consume!(self.lexer, Token::LCurly, "{")?;
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

        Ok(Ast::ClassDefinition(name.text().to_string(), members))
    }

    fn parse_op_implementation(&mut self) -> Result<Ast, String> {
        consume!(self.lexer, Token::Op, "op")?;
        let op = self
            .lexer
            .next()
            .map(|t| t.text().to_string())
            .map(|text| match text.as_str() {
                "add" => Operator::Add,
                "dot" => Operator::Dot,
                _ => panic!("Unknown operator function {}", text),
            })
            .unwrap();
        consume!(self.lexer, Token::LParan, "(")?;
        let args = self.parse_args()?;
        consume!(self.lexer, Token::LCurly, "{")?;
        let body = self.parse_stmts()?;

        Ok(Ast::OperatorImplementation(
            op,
            args.iter().map(|a| a.to_string()).collect(),
            body,
        ))
    }

    fn parse_import_statement(&mut self) -> Result<Ast, String> {
        consume!(self.lexer, Token::Import, "import")?;
        let mut tokens = Vec::new();

        while let Some(token) = self.lexer.peek() {
            match token {
                Token::Dot(data) | Token::Identifier(data) => tokens.push(data.value.to_string()),
                _ => break,
            }
            self.lexer.next();
        }

        Ok(Ast::ImportStatement(tokens.join("")))
    }

    fn parse_export_statement(&mut self) -> Result<Ast, String> {
        consume!(self.lexer, Token::Export, "export")?;

        if let Some(token) = self.lexer.peek() {
            let ast = match token {
                Token::ClassIdentifier(data) => {
                    Ast::Expression(Expression::ClassIdentifier(data.value.to_string()))
                }
                Token::Var(_) => self.parse_var_definition()?,
                Token::Class(_) => self.parse_class_definition()?,
                Token::Fn(_) => self.parse_fn_definition()?,
                Token::Identifier(data) => {
                    Ast::Expression(Expression::Identifier(data.value.to_string()))
                }
                _ => panic!("Expected either a class, variable or function"),
            };

            Ok(Ast::ExportStatement(Box::new(ast)))
        } else {
            panic!("expected either a class or a variable");
        }
    }

    fn parse_return_statement(&mut self) -> Result<Ast, String> {
        let prev_token = consume!(self.lexer, Token::Return, "return")?;

        let value = if let Some(Token::SemiColon(_)) = self.lexer.peek() {
            Expression::Null
        } else {
            self.lexer.reset_peek();
            self.parse_expr(Some(prev_token))?
        };

        Ok(Ast::ReturnStatement(value))
    }

    fn parse_documentation(&mut self) -> Result<Ast, String> {
        let mut lines = Vec::new();
        let mut line = Vec::new();

        consume!(self.lexer, Token::TripleSlash, "///").unwrap();

        while let Some(token) = self.lexer.next() {
            match token {
                Token::NewLine(_) => {
                    lines.push(line.join(" "));
                    line = Vec::new();
                    if let Some(Token::TripleSlash(_)) = self.lexer.peek() {
                        self.lexer.next();
                    } else {
                        break;
                    }
                },
                _ => {
                    line.push(token.text().to_string())
                }
            }
        }

        dbg!(&lines);

        Ok(Ast::Documentation(lines))
    }

    fn parse_var_definition(&mut self) -> Result<Ast, String> {
        consume!(self.lexer, Token::Var, "var")?;
        let ident = consume!(self.lexer, Token::Identifier, "identifier")?;
        let value = if let Some(Token::SemiColon(_)) = self.lexer.peek() {
            Expression::Null
        } else {
            let prev_token = consume!(self.lexer, Token::Equal, "=")?;
            self.parse_expr(Some(prev_token))?
        };

        Ok(Ast::VariableDefinition(ident.text().to_string(), value))
    }

    fn parse_stmts(&mut self) -> Result<Vec<Ast>, String> {
        let mut stmts = Vec::new();
        let mut depth = 0;

        while let Some(token) = self.lexer.peek() {
            let ast = match token {
                Token::Comment(_) => {
                    while let Some(token) = self.lexer.next() {
                        match token {
                            Token::NewLine(_) => break,
                            _ => {}
                        }
                    }
                    continue;
                }
                Token::TripleSlash(_) => self.parse_documentation(),
                Token::Return(_) => self.parse_return_statement(),
                Token::Break(_) => Ok(Ast::BreakStatement),
                Token::Continue(_) => Ok(Ast::ContinueStatement),
                Token::While(_) => self.parse_while_statement(),
                Token::Class(_) => self.parse_class_definition(),
                Token::Var(_) => self.parse_var_definition(),
                Token::Op(_) => self.parse_op_implementation(),
                Token::Fn(_) => self.parse_fn_definition(),
                Token::Static(_) => self.parse_static_fn_definition(),
                Token::Import(_) => self.parse_import_statement(),
                Token::Export(_) => self.parse_export_statement(),
                Token::If(_) => self.parse_if(),
                Token::ClassIdentifier(_) => {
                    if let Some(token) = self.lexer.peek() {
                        match token {
                            Token::LCurly(_) => {
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
                Token::Identifier(_) => {
                    if let Some(token) = self.lexer.peek() {
                        match token {
                            Token::Equal(_) => self.parse_var_assignment(),
                            Token::Dot(_) | Token::DoubleColon(_) => {
                                Ok(Ast::Expression(self.parse_expr(None)?))
                            },
                            _ => self.parse_expr(None).map(|x| Ast::Expression(x)),
                        }
                    } else {
                        unreachable!()
                    }
                }
                Token::RCurly(_) => {
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

    pub fn set_source(&mut self, path: PathBuf, source: &'a str) {
        self.path = path;
        self.lexer = itertools::multipeek(Lexer::new(source));
    }

    pub fn parse(&'a mut self) -> Program {
        Program {
            path: self.path.clone(),
            stmts: self.parse_stmts().unwrap(),
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

    fn expect(code: &str, ast: Ast) {
        let mut parser = Parser::new();
        parser.lexer = itertools::multipeek(Lexer::new(code));
        let prog = parser.parse();
        assert_eq!(prog.stmts, vec![ast]);
    }

    #[test]
    pub fn if_stmt() {
        expect(
            r#"if true {}"#,
            IfStatement(vec![(Expression::BooleanLiteral(true), vec![])]),
        );
    }

    #[test]
    pub fn if_stmt_with_else() {
        expect(
            r#"if true {} else {}"#,
            IfStatement(vec![
                (Expression::BooleanLiteral(true), vec![]),
                (Expression::BooleanLiteral(true), vec![]),
            ]),
        );
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
                Expression::BooleanLiteral(true),
                vec![
                    IfStatement(vec![(Expression::BooleanLiteral(true), vec![])]),
                    Expression(Expression::PostOp(
                        Box::new(Expression::Identifier("print".into())),
                        "()".into(),
                        Some(Box::new(Expression::ArrayLiteral(vec![]))),
                    )),
                ],
            ),
        );
    }
}
