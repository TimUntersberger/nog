use std::{collections::HashMap, iter::Peekable};

use itertools::Itertools;
use pratt::{Affix, Associativity, PrattParser, Precedence};

use super::{
    ast::Ast,
    expression::Expression,
    lexer::Lexer,
    operator::Operator,
    parser::{ParseError, Parser},
    token::{Token, TokenKind},
};

pub struct ExprParser<'a> {
    pub prev_token: Token,
    pub source: &'a str,
}

impl<'a> Default for ExprParser<'a> {
    fn default() -> Self {
        Self::new("")
    }
}

impl<'a> ExprParser<'a> {
    fn text(&self, token: &Token) -> &'a str {
        if token.0 == TokenKind::StringLiteral {
            let loc = token.1.start + 1..token.1.end - 1;
            &self.source[loc]
        } else {
            &self.source[token.1.clone()]
        }
    }
    fn new(source: &'a str) -> Self {
        Self {
            prev_token: Default::default(),
            source,
        }
    }
}

fn consume<I: Iterator<Item = Token>>(
    iter: &mut Peekable<&mut I>,
    kind: TokenKind,
) -> Result<Token, ParseError> {
    if let Some(token) = iter.next() {
        if token.0 == kind {
            Ok(token)
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

fn parse_inside_curlies<'a, I: Iterator<Item = Token>>(
    parser: &mut ExprParser,
    rest: &mut Peekable<&mut I>,
) -> HashMap<String, Expression> {
    let mut fields = HashMap::new();

    while let Some(token) = rest.peek() {
        match token.0 {
            TokenKind::NewLine => {
                rest.next();
                continue;
            }
            TokenKind::RCurly => {
                rest.next();
                break;
            }
            _ => {}
        };

        let token = token.clone();

        let ident = if let Some(token) = rest.next() {
            match token.0 {
                TokenKind::Identifier | TokenKind::StringLiteral => Ok(parser.text(&token)),
                _ => Err(ParseError::Unknown(token.1)),
            }
        } else {
            Err(ParseError::Unknown(token.1.clone()))
        }
        .unwrap();

        consume(rest, TokenKind::Colon).unwrap();

        let mut tokens = Vec::new();
        let mut depth = 0;

        while let Some(token) = rest.peek() {
            match token.0 {
                TokenKind::NewLine => continue,
                TokenKind::LCurly => depth += 1,
                TokenKind::RCurly => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1
                }
                TokenKind::Comma => break,
                _ => {}
            }

            tokens.push(rest.next().unwrap());
        }

        rest.next();

        let value = parser.parse(&mut tokens.into_iter()).unwrap();

        fields.insert(ident.to_string(), value);
    }

    fields
}

fn parse_object_literal<I: Iterator<Item = Token>>(
    parser: &mut ExprParser,
    rest: &mut Peekable<&mut I>,
) -> HashMap<String, Expression> {
    consume(rest, TokenKind::LCurly).unwrap();
    parse_inside_curlies(parser, rest)
}

fn parse_inside_brackets(
    parser: &mut ExprParser,
    rest: &mut dyn Iterator<Item = Token>,
) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut depth = 0;

    parser.prev_token = (TokenKind::LBracket, 0..0).into();

    while let Some(token) = rest.next() {
        match token.0 {
            TokenKind::LBracket => {
                depth += 1;
            }
            TokenKind::RBracket => {
                if depth == 0 {
                    parser.prev_token = token;
                    break;
                }
                depth -= 1;
            }
            _ => {}
        }
        tokens.push(token.clone());
    }
    tokens
}

fn parse_arg_list(
    parser: &mut ExprParser,
    rest: &mut dyn Iterator<Item = Token>,
) -> Vec<Expression> {
    let mut list = Vec::new();
    let mut arg = Vec::new();
    let mut depth = 0;
    while let Some(token) = rest.next() {
        match &token.0 {
            TokenKind::LParan | TokenKind::LBracket | TokenKind::LCurly => {
                depth += 1;
            }
            TokenKind::RParan | TokenKind::RBracket | TokenKind::RCurly => {
                depth -= 1;
            }
            TokenKind::Comma => {
                parser.prev_token = token.clone();
                if depth == 0 && !arg.is_empty() {
                    list.push(parser.parse(&mut arg.clone().into_iter()).unwrap());
                    parser.prev_token = token.clone();
                    arg.clear();
                    continue;
                }
            }
            _ => {}
        }
        arg.push(token.clone());
    }
    if !arg.is_empty() {
        list.push(parser.parse(&mut arg.into_iter()).unwrap());
    }
    list
}

fn parse_arrow_fn<T: Iterator<Item = Token>>(
    parser: &mut ExprParser,
    rest: &mut Peekable<T>,
    arg_names: Vec<String>,
) -> Result<Expression, ParseError> {
    rest.next();

    // If next token curly then treat it as code block, else parse expression
    if let Some(token) = rest.peek() {
        if token.0 == TokenKind::LCurly {
            rest.next();
            let mut level = 0;
            let mut body_tokens = Vec::new();
            while let Some(token) = rest.next() {
                match token.0 {
                    TokenKind::LCurly => level += 1,
                    TokenKind::RCurly => {
                        if level == 0 {
                            break;
                        } else {
                            level -= 1;
                        }
                    }
                    _ => body_tokens.push(token),
                };
            }
            
            let source = body_tokens.iter().map(|t| &parser.source[t.1.clone()]).join(" ");
            let mut parser = Parser::new();
            parser.set_source(
                "".into(),
                &source,
                0, //TODO: body_tokens.iter().next().unwrap_or_default().1.end,
            );
            return Ok(Expression::ArrowFunction(
                arg_names,
                parser.parse().unwrap().stmts,
            ));
        } else {
            let mut tokens = Vec::new();
            while let Some(token) = rest.next() {
                tokens.push(token);
            }
            return Ok(Expression::ArrowFunction(
                arg_names,
                vec![Ast::ReturnStatement(parser.parse(&mut tokens.into_iter())?)],
            ));
        }
    }

    todo!()
}

fn parse_inside_parenthesis(
    parser: &mut ExprParser,
    rest: &mut dyn Iterator<Item = Token>,
) -> Vec<Expression> {
    let mut tokens = Vec::new();
    let mut depth = 0;

    parser.prev_token = (TokenKind::LParan, 0..0).into();

    while let Some(token) = rest.next() {
        match token.0 {
            TokenKind::LParan => {
                parser.prev_token = token.clone();
                depth += 1;
            }
            TokenKind::RParan => {
                if depth == 0 {
                    let res = parse_arg_list(parser, &mut tokens.into_iter());
                    parser.prev_token = token;
                    return res;
                }
                depth -= 1;
            }
            _ => {}
        }
        tokens.push(token.clone());
    }
    vec![]
}

impl<'a, I> PrattParser<I> for ExprParser<'a>
where
    I: Iterator<Item = Token> + std::fmt::Debug,
{
    type Error = ParseError;
    type Output = Expression;
    type Input = Token;

    fn led(
        &mut self,
        head: Self::Input,
        tail: &mut Peekable<&mut I>,
        info: Affix,
        lhs: Self::Output,
    ) -> Result<Self::Output, pratt::PrattError<Self::Input, Self::Error>> {
        self.prev_token = head.clone();
        match info {
            Affix::Infix(precedence, associativity) => {
                let precedence = Precedence(precedence.0 * 10);
                let rhs = match associativity {
                    Associativity::Left => self.parse_input(tail, precedence),
                    Associativity::Right => self.parse_input(tail, Precedence(precedence.0 - 1)),
                    Associativity::Neither => self.parse_input(tail, Precedence(precedence.0 + 1)),
                };
                self.infix(lhs, head, rhs?, tail)
                    .map_err(pratt::PrattError::UserError)
            }
            Affix::Postfix(_) => self
                .postfix(lhs, head, tail)
                .map_err(pratt::PrattError::UserError),
            Affix::Nilfix => Err(pratt::PrattError::UnexpectedNilfix(head)),
            Affix::Prefix(_) => Err(pratt::PrattError::UnexpectedPrefix(head)),
        }
    }

    fn query(
        &mut self,
        token: &Self::Input,
        _: &mut Peekable<&mut I>,
    ) -> Result<Affix, Self::Error> {
        let res = match token.0 {
            TokenKind::Plus | TokenKind::Minus => match &self.prev_token.0 {
                TokenKind::Error
                | TokenKind::Star
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::PlusEqual
                | TokenKind::MinusEqual
                | TokenKind::StarEqual
                | TokenKind::SlashEqual
                | TokenKind::LParan
                | TokenKind::LBracket
                | TokenKind::Equal => Affix::Prefix(Precedence(12)),
                _ => Affix::Infix(Precedence(3), Associativity::Left),
            },
            TokenKind::Star | TokenKind::Slash => Affix::Infix(Precedence(4), Associativity::Left),
            TokenKind::GT
            | TokenKind::GTE
            | TokenKind::LT
            | TokenKind::LTE
            | TokenKind::EQ
            | TokenKind::NEQ => Affix::Infix(Precedence(9), Associativity::Left),
            TokenKind::Dot => Affix::Infix(Precedence(11), Associativity::Left),
            TokenKind::And => Affix::Infix(Precedence(8), Associativity::Left),
            TokenKind::Or => Affix::Infix(Precedence(8), Associativity::Left),
            TokenKind::DoubleColon => Affix::Infix(Precedence(11), Associativity::Left),
            TokenKind::Equal => Affix::Infix(Precedence(1), Associativity::Neither),
            TokenKind::PlusEqual => Affix::Infix(Precedence(1), Associativity::Neither),
            TokenKind::MinusEqual => Affix::Infix(Precedence(1), Associativity::Neither),
            TokenKind::StarEqual => Affix::Infix(Precedence(1), Associativity::Neither),
            TokenKind::SlashEqual => Affix::Infix(Precedence(1), Associativity::Neither),
            TokenKind::LParan | TokenKind::LBracket => match &self.prev_token.0 {
                TokenKind::LCurly
                | TokenKind::RCurly
                | TokenKind::LParan
                | TokenKind::LBracket
                | TokenKind::Return
                | TokenKind::Error
                | TokenKind::Equal
                | TokenKind::PlusEqual
                | TokenKind::MinusEqual
                | TokenKind::StarEqual
                | TokenKind::SlashEqual
                | TokenKind::Comma => Affix::Nilfix,
                _ => Affix::Postfix(Precedence(10)),
            },
            TokenKind::LCurly => match &self.prev_token.0 {
                TokenKind::ClassIdentifier => Affix::Postfix(Precedence(10)),
                _ => Affix::Nilfix,
            },
            TokenKind::MinusMinus | TokenKind::PlusPlus => Affix::Postfix(Precedence(10)),
            TokenKind::ExclamationMark => Affix::Prefix(Precedence(10)),
            TokenKind::Arrow => Affix::Nilfix,
            TokenKind::RParan => Affix::Nilfix,
            TokenKind::Hash => Affix::Nilfix,
            TokenKind::RCurly => Affix::Nilfix,
            TokenKind::RBracket => Affix::Nilfix,
            TokenKind::Import => Affix::Nilfix,
            TokenKind::NewLine => Affix::Nilfix,
            TokenKind::NumberLiteral => Affix::Nilfix,
            TokenKind::StringLiteral => Affix::Nilfix,
            TokenKind::ClassIdentifier => Affix::Nilfix,
            TokenKind::BooleanLiteral => Affix::Nilfix,
            TokenKind::Identifier => Affix::Nilfix,
            TokenKind::Null => Affix::Nilfix,
            _ => unreachable!("{:?}", token),
        };

        match token.0 {
            TokenKind::Star | TokenKind::Plus | TokenKind::Minus => {}
            _ => {}
        }

        Ok(res)
    }
    fn primary(
        &mut self,
        token: Self::Input,
        rest: &mut Peekable<&mut I>,
    ) -> Result<Self::Output, Self::Error> {
        self.prev_token = token.clone();
        let text = self.text(&token).to_string();
        Ok(match token.0 {
            TokenKind::NumberLiteral => Expression::NumberLiteral(text),
            TokenKind::StringLiteral => {
                let raw = text
                    .clone()
                    .replace("\\\\", "\\")
                    .replace("\\\"", "\"")
                    .replace("\\r", "\r")
                    .replace("\\n", "\n");
                Expression::StringLiteral(raw)
            }
            TokenKind::BooleanLiteral => Expression::BooleanLiteral(text),
            TokenKind::Null => Expression::Null,
            TokenKind::LBracket => {
                let mut args = Vec::new();
                let mut arg_tokens = Vec::new();

                while let Some(next) = rest.next() {
                    if let TokenKind::RBracket = next.0 {
                        if !arg_tokens.is_empty() {
                            args.push(self.parse(&mut arg_tokens.clone().into_iter())?);
                        }
                        break;
                    } else if let TokenKind::Comma = next.0 {
                        args.push(self.parse(&mut arg_tokens.clone().into_iter())?);
                        arg_tokens.clear();
                    } else {
                        if let TokenKind::LBracket = next.0 {
                            args.push(self.primary(next, rest)?);
                            arg_tokens.clear();
                        } else {
                            arg_tokens.push(next);
                        }
                    }
                }

                Expression::ArrayLiteral(args)
            }
            TokenKind::Hash => {
                let fields = parse_object_literal(self, rest);
                Expression::ObjectLiteral(fields)
            }
            TokenKind::ClassIdentifier => Expression::ClassIdentifier(text),
            TokenKind::Identifier => {
                if let Some(token) = rest.peek().cloned() {
                    match token.0 {
                        TokenKind::Arrow => {
                            return parse_arrow_fn(self, rest, vec![text]);
                        }
                        _ => {}
                    }
                }
                Expression::Identifier(text)
            }
            TokenKind::LParan => {
                let mut depth = 0;
                let mut tokens = Vec::new();

                while let Some(token) = rest.next() {
                    match token.0 {
                        TokenKind::LParan => {
                            depth += 1;
                        }
                        TokenKind::RParan => {
                            if depth == 0 {
                                break;
                            } else {
                                depth -= 1;
                            }
                        }
                        _ => {
                            tokens.push(token);
                        }
                    }
                }

                if let Some(token) = rest.peek() {
                    match token.0 {
                        TokenKind::Arrow => {
                            return parse_arrow_fn(
                                self,
                                rest,
                                tokens.iter().map(|t| self.text(t).to_string()).collect(),
                            );
                        }
                        _ => {
                            //TODO: handle error
                            return self.parse(&mut tokens.into_iter()).map_err(|x| x.into());
                        }
                    }
                } else {
                    //TODO: handle error
                    return self.parse(&mut tokens.into_iter()).map_err(|x| x.into());
                }
            }
            _ => unreachable!(),
        })
    }
    fn infix(
        &mut self,
        lhs: Self::Output,
        token: Self::Input,
        rhs: Self::Output,
        _: &mut Peekable<&mut I>,
    ) -> Result<Self::Output, Self::Error> {
        Ok(Expression::BinaryOp(
            Box::new(lhs),
            token.0.into(),
            Box::new(rhs),
        ))
    }
    fn prefix(
        &mut self,
        token: Self::Input,
        rhs: Self::Output,
        _: &mut Peekable<&mut I>,
    ) -> Result<Self::Output, Self::Error> {
        Ok(Expression::PreOp(token.0.into(), Box::new(rhs)))
    }
    fn postfix(
        &mut self,
        lhs: Self::Output,
        token: Self::Input,
        rest: &mut Peekable<&mut I>,
    ) -> Result<Self::Output, Self::Error> {
        match token.0 {
            TokenKind::LBracket => {
                let tokens = parse_inside_brackets(self, rest);
                Ok(Expression::PostOp(
                    Box::new(lhs),
                    Operator::Index,
                    Some(Box::new(self.parse(&mut tokens.into_iter()).unwrap())),
                ))
            }
            TokenKind::LParan => {
                let args = parse_inside_parenthesis(self, rest);
                Ok(Expression::PostOp(
                    Box::new(lhs),
                    Operator::Call,
                    Some(Box::new(Expression::ArrayLiteral(args))),
                ))
            }
            TokenKind::LCurly => {
                let fields = parse_inside_curlies(self, rest);
                Ok(Expression::PostOp(
                    Box::new(lhs),
                    Operator::Constructor,
                    Some(Box::new(Expression::ObjectLiteral(fields))),
                ))
            }
            TokenKind::MinusMinus => Ok(Expression::PostOp(Box::new(lhs), token.0.into(), None)),
            TokenKind::PlusPlus => Ok(Expression::PostOp(Box::new(lhs), token.0.into(), None)),
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::ExprParser;
    use crate::{ast::Ast, expression::Expression::*};
    use crate::{expression::Expression, lexer::Lexer, operator::Operator, token::Token};
    use logos::Logos;
    use pratt::PrattParser;
    use std::collections::HashMap;

    fn parse(input: &str) -> Expression {
        ExprParser::new(input)
            .parse(&mut Lexer::new(input, 0))
            .unwrap()
    }

    fn binary(lhs: Expression, op: &str, rhs: Expression) -> Expression {
        Expression::BinaryOp(
            Box::new(lhs),
            Operator::from_str(op).unwrap(),
            Box::new(rhs),
        )
    }

    fn post(lhs: Expression, op: &str, value: Option<Expression>) -> Expression {
        Expression::PostOp(
            Box::new(lhs),
            Operator::from_str(op).unwrap(),
            value.map(|x| Box::new(x)),
        )
    }

    fn pre(op: &str, lhs: Expression) -> Expression {
        Expression::PreOp(Operator::from_str(op).unwrap(), Box::new(lhs))
    }

    fn array(items: Vec<Expression>) -> Expression {
        Expression::ArrayLiteral(items)
    }

    fn object(fields: HashMap<&str, Expression>) -> Expression {
        Expression::ObjectLiteral(
            fields
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn ident(name: &str) -> Expression {
        Expression::Identifier(name.into())
    }

    fn class(name: &str) -> Expression {
        Expression::ClassIdentifier(name.into())
    }

    fn string(name: &str) -> Expression {
        Expression::StringLiteral(name.into())
    }

    fn call_op(lhs: Expression, args: Vec<Expression>) -> Expression {
        post(lhs, "()", Some(array(args)))
    }

    fn index_op(lhs: Expression, rhs: Expression) -> Expression {
        post(lhs, "[]", Some(rhs))
    }

    fn number(x: i32) -> Expression {
        Expression::NumberLiteral(x.to_string())
    }

    fn boolean(x: bool) -> Expression {
        Expression::BooleanLiteral(x.to_string())
    }

    fn add_op(lhs: Expression, rhs: Expression) -> Expression {
        binary(lhs, "+", rhs)
    }

    fn dot_op(lhs: Expression, rhs: Expression) -> Expression {
        binary(lhs, ".", rhs)
    }

    fn instance(name: &str, fields: HashMap<&str, Expression>) -> Expression {
        post(
            class(name),
            "{}",
            Some(Expression::ObjectLiteral(
                fields
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect(),
            )),
        )
    }

    #[test]
    fn math_expr1() {
        assert_eq!(parse("1 + 2"), binary(number(1), "+", number(2),));
    }

    #[test]
    fn math_expr2() {
        assert_eq!(
            parse("1 + 2 - 3"),
            binary(binary(number(1), "+", number(2),), "-", number(3))
        );
    }

    #[test]
    fn math_expr3() {
        assert_eq!(
            parse("1 + 2 * 3"),
            binary(number(1), "+", binary(number(2), "*", number(3),),)
        );
    }

    #[test]
    fn math_expr4() {
        assert_eq!(
            parse("(1 + 2) * 3"),
            binary(binary(number(1), "+", number(2)), "*", number(3),)
        );
    }

    #[test]
    fn op_plus() {
        assert_eq!(
            parse(r#"a + " world""#),
            binary(ident("a"), "+", string(" world"),)
        );
    }

    #[test]
    fn op_call() {
        assert_eq!(parse(r#"test()"#), call_op(ident("test"), vec![]),);
    }

    #[test]
    fn op_call_with_args() {
        assert_eq!(
            parse(r#"print(hello, world)"#),
            call_op(ident("print"), vec![ident("hello"), ident("world")]),
        );
    }

    #[test]
    fn op_call_with_expr_args() {
        assert_eq!(
            parse(r#"print(get_message(), "string", 2, (1 + 3), true)"#),
            call_op(
                ident("print"),
                vec![
                    call_op(ident("get_message"), vec![]),
                    string("string"),
                    number(2),
                    add_op(number(1), number(3)),
                    boolean(true)
                ]
            ),
        );
    }

    #[test]
    fn op_add() {
        assert_eq!(
            parse(r#""Hello " + test()"#),
            add_op(string("Hello "), call_op(ident("test"), vec![]))
        );
    }

    #[test]
    fn op_dot() {
        assert_eq!(
            parse(r"account.username"),
            dot_op(ident("account"), ident("username"),)
        );
    }

    #[test]
    fn array_literal() {
        assert_eq!(parse(r"[1, 2]"), array(vec![number(1), number(2)]));
    }

    #[test]
    fn nested_array_literal() {
        assert_eq!(
            parse(r"[1, [2, 3]]"),
            array(vec![number(1), array(vec![number(2), number(3)])])
        );
    }

    #[test]
    fn op_call_with_array_arg() {
        assert_eq!(
            parse(r"print([this, other])"),
            call_op(
                ident("print"),
                vec![array(vec![ident("this"), ident("other")])]
            )
        );
    }

    #[test]
    fn op_add_classes() {
        assert_eq!(
            parse(r"User{} + User{}"),
            add_op(
                instance("User", HashMap::new()),
                instance("User", HashMap::new())
            )
        );
    }

    #[test]
    fn op_dot_class_with_instance_fn_call() {
        assert_eq!(
            parse(r"User{}.hello()"),
            call_op(
                dot_op(instance("User", HashMap::new()), ident("hello")),
                vec![]
            )
        );
    }

    #[test]
    fn object_literal() {
        assert_eq!(parse(r"#{}"), object(HashMap::new()));
    }

    #[test]
    fn object_literal_with_fields() {
        assert_eq!(
            parse(r#"#{ username: "test" }"#),
            object(hashmap! { "username" => string("test") }),
        );
    }

    #[test]
    fn nested_dot_operator_with_fn_call() {
        assert_eq!(
            parse(r#"user.names.push(1)"#),
            call_op(
                dot_op(dot_op(ident("user"), ident("names")), ident("push")),
                vec![number(1)]
            )
        );
    }

    #[test]
    fn static_class_fn_call() {
        assert_eq!(
            parse(r#"User.new()"#),
            call_op(dot_op(class("User"), ident("new")), vec![])
        );
    }

    #[test]
    fn namespace_fn_call() {
        assert_eq!(
            parse(r#"user.call()"#),
            call_op(dot_op(ident("user"), ident("call")), vec![])
        );
    }

    #[test]
    fn nested_namespace_op() {
        assert_eq!(
            parse(r#"user.functions.call"#),
            dot_op(dot_op(ident("user"), ident("functions")), ident("call"))
        );
    }

    #[test]
    fn nested_namespace_op_with_fn_call() {
        assert_eq!(
            parse(r#"user.functions.call()"#),
            call_op(
                dot_op(dot_op(ident("user"), ident("functions")), ident("call")),
                vec![]
            )
        );
    }

    #[test]
    fn arrow_fn() {
        assert_eq!(parse(r#"() => {}"#), ArrowFunction(vec![], vec![]));
    }

    #[test]
    fn arrow_fn_with_args() {
        assert_eq!(
            parse(r#"(test) => {}"#),
            ArrowFunction(vec!["test".into()], vec![])
        );
    }

    #[test]
    fn arrow_fn_with_body() {
        assert_eq!(
            parse(
                r#"() => {
                    print(1);
                }"#
            ),
            ArrowFunction(
                vec![],
                vec![Ast::Expression(call_op(ident("print"), vec![number(1)]))]
            )
        );
    }

    //TODO: This test never stops running
    #[test]
    #[ignore]
    fn object_literal_with_arrow_fn() {
        assert_eq!(
            parse(
                r#"#{
                    f: () => {
                        print("hello world");
                    }
                }"#
            ),
            object(hashmap! {
                "f" => ArrowFunction(
                    vec![],
                    vec![Ast::FunctionCall(
                        "print".into(),
                        vec![string("hello world")]
                    )]
                )
            })
        );
    }

    #[test]
    fn nested_fn_call() {
        assert_eq!(
            parse(r#"f()()"#),
            call_op(call_op(ident("f"), vec![]), vec![])
        );
    }

    #[test]
    fn index_operator_0() {
        assert_eq!(parse(r#"array[0]"#), index_op(ident("array"), number(0)));
    }

    #[test]
    fn index_operator_field() {
        assert_eq!(
            parse(r#"object["name"]"#),
            index_op(ident("object"), string("name"))
        );
    }

    #[test]
    fn advanced_index_operator_usage1() {
        assert_eq!(
            parse(r#"f()[0]"#),
            index_op(call_op(ident("f"), vec![]), number(0))
        );
    }

    #[test]
    fn advanced_index_operator_usage2() {
        assert_eq!(
            parse(r#"f()[0]()"#),
            call_op(index_op(call_op(ident("f"), vec![]), number(0)), vec![])
        );
    }

    #[test]
    fn advanced_index_operator_usage3() {
        assert_eq!(
            parse(r#"print(f()[0]())"#),
            call_op(
                ident("print"),
                vec![call_op(
                    index_op(call_op(ident("f"), vec![]), number(0)),
                    vec![]
                )]
            )
        );
    }

    #[test]
    fn advanced_index_operator_usage4() {
        assert_eq!(
            parse(r#"f[0][0]"#),
            index_op(index_op(ident("f"), number(0)), number(0))
        );
    }

    #[test]
    fn constructor_op() {
        assert_eq!(
            parse("User{}"),
            post(class("User"), "{}", Some(object(hashmap! {})))
        );
    }

    #[test]
    fn constructor_op_with_namespace() {
        assert_eq!(
            parse("lib.User{}"),
            post(
                dot_op(ident("lib"), class("User")),
                "{}",
                Some(object(hashmap! {}))
            )
        );
    }

    #[test]
    fn constructor_op_with_fields() {
        assert_eq!(
            parse("User{test: 1}"),
            post(
                class("User"),
                "{}",
                Some(object(hashmap! {"test" => number(1)}))
            )
        );
    }

    #[test]
    fn minus_minus() {
        assert_eq!(parse("test--"), post(ident("test"), "--", None));
    }

    #[test]
    fn plus_plus() {
        assert_eq!(parse("test++"), post(ident("test"), "++", None));
    }

    #[test]
    fn not_op() {
        assert_eq!(parse("!true"), pre("!", boolean(true)));
    }

    #[test]
    fn negative_op() {
        assert_eq!(parse("-2"), pre("-", number(2)));
    }

    #[test]
    fn function_method() {
        assert_eq!(
            parse("range(10).for_each(i => print(i))"),
            call_op(
                binary(
                    call_op(ident("range"), vec![number(10)]),
                    ".",
                    ident("for_each")
                ),
                vec![Expression::ArrowFunction(
                    vec!["i".into()],
                    vec![Ast::ReturnStatement(call_op(
                        ident("print"),
                        vec![ident("i")]
                    ))]
                )]
            )
        );
    }

    #[test]
    fn negative_op_inside_math() {
        assert_eq!(
            parse("2 * -2 + 1"),
            binary(binary(number(2), "*", pre("-", number(2))), "+", number(1))
        );
    }

    #[test]
    fn logic_operators() {
        let ops = vec![">", "<", ">=", "<=", "==", "!=", "&&", "||"];
        for op in &ops {
            assert_eq!(
                parse(&format!("1 {} 2", op)),
                binary(number(1), op, number(2),)
            );
        }
    }
}
