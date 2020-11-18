use std::{collections::HashMap, iter::Peekable};

use pratt::{Affix, Associativity, PrattParser, Precedence};

use super::{expression::Expression, token::Token};

pub struct ExprParser;

fn parse_object_literal<'a, I: Iterator<Item = Token<'a>>>(
    parser: &mut ExprParser,
    rest: &mut Peekable<&mut I>,
) -> HashMap<String, Expression> {
    let mut fields = HashMap::new();

    consume!(rest, Token::LCurly, "{", true).unwrap();

    while let Some(token) = rest.peek() {
        match token {
            Token::NewLine(_) => {
                rest.next();
                continue;
            }
            Token::RCurly(_) => {
                rest.next();
                break;
            }
            _ => {}
        };

        let ident = consume!(rest, Token::Identifier, "identifier").unwrap();
        consume!(rest, Token::Equal, ":").unwrap();

        let mut tokens = Vec::new();
        let mut depth = 0;

        while let Some(token) = rest.next() {
            match token {
                Token::LCurly(_) => depth += 1,
                Token::RCurly(_) => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1
                }
                Token::Comma(_) => break,
                _ => {}
            }

            tokens.push(token);
        }

        let value = parser.parse(&mut tokens.into_iter()).unwrap();

        fields.insert(ident.text().to_string(), value);
    }

    fields
}

impl<'a, I> PrattParser<I> for ExprParser
where
    I: Iterator<Item = Token<'a>>,
{
    type Error = pratt::NoError;
    type Output = Expression;
    type Input = Token<'a>;

    fn query(&mut self, token: &Self::Input, _: &mut Peekable<&mut I>) -> pratt::Result<Affix> {
        Ok(match token {
            Token::Symbol(data) => match data.value {
                "+" => Affix::Infix(Precedence(3), Associativity::Left),
                "-" => Affix::Infix(Precedence(3), Associativity::Left),
                "*" => Affix::Infix(Precedence(4), Associativity::Left),
                "/" => Affix::Infix(Precedence(4), Associativity::Left),
                _ => unreachable!(data.value),
            },
            Token::Dot(_) => Affix::Infix(Precedence(10), Associativity::Left),
            Token::DoubleColon(_) => Affix::Infix(Precedence(11), Associativity::Left),
            Token::Equal(_) => Affix::Infix(Precedence(2), Associativity::Neither),
            Token::LParan(_) => Affix::Nilfix,
            Token::RParan(_) => Affix::Nilfix,
            Token::Hash(_) => Affix::Nilfix,
            Token::LCurly(_) => Affix::Nilfix,
            Token::RCurly(_) => Affix::Nilfix,
            Token::LBracket(_) => Affix::Nilfix,
            Token::RBracket(_) => Affix::Nilfix,
            Token::NumberLiteral(_) => Affix::Nilfix,
            Token::StringLiteral(_) => Affix::Nilfix,
            Token::ClassIdentifier(_) => Affix::Nilfix,
            Token::BooleanLiteral(_) => Affix::Nilfix,
            Token::Identifier(_) => Affix::Nilfix,
            Token::Null(_) => Affix::Nilfix,
            _ => unreachable!("{:?}", token),
        })
    }
    fn primary(
        &mut self,
        token: Self::Input,
        rest: &mut Peekable<&mut I>,
    ) -> pratt::Result<Self::Output> {
        Ok(match token {
            Token::NumberLiteral(data) => Expression::NumberLiteral(data.value),
            Token::StringLiteral(data) => Expression::StringLiteral(data.value.to_string()),
            Token::BooleanLiteral(data) => Expression::BooleanLiteral(data.value),
            Token::Null(_) => Expression::Null,
            Token::LBracket(_) => {
                let mut args = Vec::new();
                let mut arg_tokens = Vec::new();

                while let Some(next) = rest.next() {
                    if let Token::RBracket(_) = next {
                        if !arg_tokens.is_empty() {
                            args.push(
                                self.parse(&mut arg_tokens.clone().into_iter())
                                    .map_err(|_| pratt::NoError)?,
                            );
                        }
                        break;
                    } else if let Token::Comma(_) = next {
                        args.push(
                            self.parse(&mut arg_tokens.clone().into_iter())
                                .map_err(|_| pratt::NoError)?,
                        );
                        arg_tokens.clear();
                    } else {
                        if let Token::LBracket(_) = next {
                            args.push(self.primary(next, rest)?);
                            arg_tokens.clear();
                        } else {
                            arg_tokens.push(next);
                        }
                    }
                }

                Expression::ArrayLiteral(args)
            }
            Token::Hash(_) => {
                let fields = parse_object_literal(self, rest);
                Expression::ObjectLiteral(fields)
            }
            Token::ClassIdentifier(data) => {
                let ident = data.value.to_string();

                match rest.peek() {
                    Some(Token::LCurly(_)) => {
                        let fields = parse_object_literal(self, rest);

                        return Ok(Expression::ClassInstantiation(ident, fields));
                    }
                    Some(Token::Dot(_)) => {}
                    _ => todo!(),
                };

                Expression::ClassIdentifier(ident)
            }
            Token::Identifier(data) => {
                let ident = data.value.to_string();
                if let Some(next) = rest.peek() {
                    match next {
                        Token::LParan(_) => {
                            rest.next();
                            let mut depth = 0;
                            let mut args = Vec::new();
                            let mut arg_tokens = Vec::new();
                            while let Some(next) = rest.next() {
                                match &next {
                                    Token::LBracket(_) => {
                                        args.push(self.primary(next, rest)?);
                                        continue;
                                    }
                                    Token::LParan(_) => depth += 1,
                                    Token::RParan(_) => {
                                        if depth == 0 {
                                            if !arg_tokens.is_empty() {
                                                args.push(
                                                    self.parse(&mut arg_tokens.clone().into_iter())
                                                        .map_err(|_| pratt::NoError)?,
                                                );
                                            }
                                            return Ok(Expression::FunctionCall(ident, args));
                                        }

                                        depth -= 1;
                                    }
                                    Token::Comma(_) => {
                                        args.push(
                                            self.parse(&mut arg_tokens.clone().into_iter())
                                                .map_err(|_| pratt::NoError)?,
                                        );
                                        arg_tokens.clear();
                                        continue;
                                    }
                                    _ => {}
                                };

                                arg_tokens.push(next);
                            }
                        }
                        _ => {}
                    }
                }
                Expression::Identifier(ident)
            }
            Token::LParan(_) => {
                let mut depth = 0;
                let mut tokens = Vec::new();

                while let Some(token) = rest.next() {
                    match token {
                        Token::LParan(_) => {
                            depth += 1;
                        }
                        Token::RParan(_) => {
                            if depth == 0 {
                                break;
                            } else {
                                depth -= 1;
                            }
                        }
                        token => {
                            tokens.push(token);
                        }
                    }
                }

                //TODO: handle error
                return self
                    .parse(&mut tokens.into_iter())
                    .map_err(|_| pratt::NoError);
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
    ) -> pratt::Result<Self::Output> {
        Ok(Expression::BinaryOp(
            Box::new(lhs),
            token.text().to_string(),
            Box::new(rhs),
        ))
    }
    fn prefix(
        &mut self,
        _token: Self::Input,
        _rhs: Self::Output,
        _: &mut Peekable<&mut I>,
    ) -> pratt::Result<Self::Output> {
        todo!();
    }
    fn postfix(
        &mut self,
        _lhs: Self::Output,
        _token: Self::Input,
        _: &mut Peekable<&mut I>,
    ) -> pratt::Result<Self::Output> {
        todo!();
    }
}

#[cfg(test)]
mod test {
    use super::ExprParser;
    use crate::expression::Expression::*;
    use crate::{expression::Expression, token::Token};
    use logos::Logos;
    use pratt::PrattParser;
    use std::collections::HashMap;

    fn parse(input: &str) -> Expression {
        ExprParser.parse(&mut Token::lexer(input)).unwrap()
    }

    fn binary(lhs: Expression, op: &str, rhs: Expression) -> Expression {
        Expression::BinaryOp(Box::new(lhs), op.into(), Box::new(rhs))
    }

    #[test]
    fn test1() {
        assert_eq!(
            parse("1 + 2"),
            binary(NumberLiteral(1), "+", NumberLiteral(2),)
        );
    }

    #[test]
    fn test2() {
        assert_eq!(
            parse("1 + 2 - 3"),
            binary(
                binary(NumberLiteral(1), "+", NumberLiteral(2),),
                "-",
                NumberLiteral(3)
            )
        );
    }

    #[test]
    fn test3() {
        assert_eq!(
            parse("1 + 2 * 3"),
            binary(
                NumberLiteral(1),
                "+",
                binary(NumberLiteral(2), "*", NumberLiteral(3),),
            )
        );
    }

    #[test]
    fn test4() {
        assert_eq!(
            parse("(1 + 2) * 3"),
            binary(
                binary(NumberLiteral(1), "+", NumberLiteral(2)),
                "*",
                NumberLiteral(3),
            )
        );
    }

    #[test]
    fn test5() {
        assert_eq!(
            parse(r#"a + " world""#),
            binary(Identifier("a".into()), "+", StringLiteral(" world".into()),)
        );
    }

    #[test]
    fn test6() {
        assert_eq!(parse(r#"test()"#), FunctionCall("test".into(), vec![]),);
    }

    #[test]
    fn test7() {
        assert_eq!(
            parse(r#"print(hello, world)"#),
            FunctionCall(
                "print".into(),
                vec![Identifier("hello".into()), Identifier("world".into())]
            ),
        );
    }

    #[test]
    fn test8() {
        assert_eq!(
            parse(r#"print(get_message(), "string", 2, (1 + 3), true)"#),
            FunctionCall(
                "print".into(),
                vec![
                    FunctionCall("get_message".into(), vec![]),
                    StringLiteral("string".into()),
                    NumberLiteral(2),
                    binary(NumberLiteral(1), "+", NumberLiteral(3)),
                    BooleanLiteral(true)
                ]
            ),
        );
    }

    #[test]
    fn test9() {
        assert_eq!(
            parse(r#""Hello " + test()"#),
            binary(
                StringLiteral("Hello ".into()),
                "+",
                FunctionCall("test".into(), vec![]),
            )
        );
    }

    #[test]
    fn test10() {
        assert_eq!(
            parse(r"account.username"),
            binary(
                Identifier("account".into()),
                ".",
                Identifier("username".into()),
            )
        );
    }

    #[test]
    fn test11() {
        assert_eq!(
            parse(r"[1, 2]"),
            ArrayLiteral(vec![NumberLiteral(1), NumberLiteral(2),])
        );
    }

    #[test]
    fn test12() {
        assert_eq!(
            parse(r"[1, [2, 3]]"),
            ArrayLiteral(vec![
                NumberLiteral(1),
                ArrayLiteral(vec![NumberLiteral(2), NumberLiteral(3),])
            ])
        );
    }

    #[test]
    fn test13() {
        assert_eq!(
            parse(r"print([this, other])"),
            FunctionCall(
                "print".into(),
                vec![ArrayLiteral(vec![
                    Identifier("this".into()),
                    Identifier("other".into()),
                ])]
            )
        );
    }

    #[test]
    fn test14() {
        assert_eq!(
            parse(r"User{} + User{}"),
            binary(
                ClassInstantiation("User".into(), HashMap::new()),
                "+",
                ClassInstantiation("User".into(), HashMap::new()),
            )
        );
    }

    #[test]
    fn test15() {
        assert_eq!(
            parse(r"User{}.hello()"),
            binary(
                ClassInstantiation("User".into(), HashMap::new()),
                ".",
                FunctionCall("hello".into(), vec![]),
            )
        );
    }

    #[test]
    fn test16() {
        assert_eq!(parse(r"#{}"), ObjectLiteral(HashMap::new()),);
    }

    #[test]
    fn test17() {
        assert_eq!(
            parse(r#"#{ username = "test" }"#),
            ObjectLiteral(hashmap! { "username".to_string() => StringLiteral("test".into()) }),
        );
    }

    #[test]
    fn test18() {
        assert_eq!(
            parse(r#"user.names.push(1)"#),
            binary(
                binary(Identifier("user".into()), ".", Identifier("names".into())),
                ".",
                FunctionCall("push".into(), vec![NumberLiteral(1)])
            ),
        );
    }

    #[test]
    fn test19() {
        assert_eq!(
            parse(r#"User.new()"#),
            binary(
                ClassIdentifier("User".into()),
                ".",
                FunctionCall("new".into(), vec![])
            ),
        );
    }

    #[test]
    fn test20() {
        assert_eq!(
            parse(r#"user::call()"#),
            binary(
                Identifier("user".into()),
                "::",
                FunctionCall("call".into(), vec![])
            ),
        );
    }

    #[test]
    fn test21() {
        assert_eq!(
            parse(r#"user::functions::call()"#),
            binary(
                binary(
                    Identifier("user".into()),
                    "::",
                    Identifier("functions".into()),
                ),
                "::",
                FunctionCall("call".into(), vec![])
            ),
        );
    }
}
