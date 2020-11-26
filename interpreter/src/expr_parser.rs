use std::{collections::HashMap, iter::Peekable};

use itertools::Itertools;
use pratt::{Affix, Associativity, PrattParser, Precedence};

use super::{
    ast::Ast,
    expression::Expression,
    lexer::Lexer,
    parser::Parser,
    token::{Token, TokenData},
};

pub struct ExprParser<'a> {
    pub prev_token: Token<'a>,
}

impl<'a> Default for ExprParser<'a> {
    fn default() -> Self {
        Self {
            prev_token: Token::Error,
        }
    }
}

fn parse_inside_curlies<'a, I: Iterator<Item = Token<'a>>>(
    parser: &mut ExprParser<'a>,
    rest: &mut Peekable<&mut I>,
) -> HashMap<String, Expression> {
    let mut fields = HashMap::new();

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
        consume!(rest, Token::Colon, ":").unwrap();

        let mut tokens = Vec::new();
        let mut depth = 0;

        while let Some(token) = rest.peek() {
            match token {
                Token::NewLine(_) => continue,
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

            tokens.push(rest.next().unwrap());
        }

        rest.next();

        let value = parser.parse(&mut tokens.into_iter()).unwrap();

        fields.insert(ident.text().to_string(), value);
    }

    fields
}

fn parse_object_literal<'a, I: Iterator<Item = Token<'a>>>(
    parser: &mut ExprParser<'a>,
    rest: &mut Peekable<&mut I>,
) -> HashMap<String, Expression> {
    consume!(rest, Token::LCurly, "{", true).unwrap();
    parse_inside_curlies(parser, rest)
}

fn parse_inside_brackets<'a>(
    parser: &mut ExprParser<'a>,
    rest: &mut dyn Iterator<Item = Token<'a>>,
) -> Vec<Token<'a>> {
    let mut tokens = Vec::new();
    let mut depth = 0;

    parser.prev_token = Token::LBracket(TokenData::default());

    while let Some(token) = rest.next() {
        match token {
            Token::LBracket(_) => {
                depth += 1;
            }
            Token::RBracket(_) => {
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

fn parse_arg_list<'a>(
    parser: &mut ExprParser<'a>,
    rest: &mut dyn Iterator<Item = Token<'a>>,
) -> Vec<Expression> {
    let mut list = Vec::new();
    let mut arg = Vec::new();
    let mut depth = 0;
    while let Some(token) = rest.next() {
        match token {
            Token::LParan(_) | Token::LBracket(_) | Token::LCurly(_) => {
                depth += 1;
            }
            Token::RParan(_) | Token::RBracket(_) | Token::RCurly(_) => {
                depth -= 1;
            }
            Token::Comma(_) => {
                if depth == 0 && !arg.is_empty() {
                    parser.prev_token = token;
                    list.push(parser.parse(&mut arg.clone().into_iter()).unwrap());
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

fn parse_inside_parenthesis<'a>(
    parser: &mut ExprParser<'a>,
    rest: &mut dyn Iterator<Item = Token<'a>>,
) -> Vec<Expression> {
    let mut tokens = Vec::new();
    let mut depth = 0;

    parser.prev_token = Token::LParan(TokenData::default());

    while let Some(token) = rest.next() {
        match token {
            Token::LParan(_) => {
                parser.prev_token = token.clone();
                depth += 1;
            }
            Token::RParan(_) => {
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
    I: Iterator<Item = Token<'a>>,
{
    type Error = pratt::NoError;
    type Output = Expression;
    type Input = Token<'a>;

    fn query(&mut self, token: &Self::Input, _: &mut Peekable<&mut I>) -> pratt::Result<Affix> {
        let res = match token {
            Token::Symbol(data) => match data.value {
                "+" => Affix::Infix(Precedence(3), Associativity::Left),
                "-" => Affix::Infix(Precedence(3), Associativity::Left),
                "*" => Affix::Infix(Precedence(4), Associativity::Left),
                "/" => Affix::Infix(Precedence(4), Associativity::Left),
                _ => unreachable!(data.value),
            },
            Token::GT(_)
            | Token::GTE(_)
            | Token::LT(_)
            | Token::LTE(_)
            | Token::EQ(_)
            | Token::NEQ(_) => Affix::Infix(Precedence(2), Associativity::Left),
            Token::Dot(_) => Affix::Infix(Precedence(11), Associativity::Left),
            Token::And(_) => Affix::Infix(Precedence(8), Associativity::Left),
            Token::Or(_) => Affix::Infix(Precedence(8), Associativity::Left),
            Token::DoubleColon(_) => Affix::Infix(Precedence(11), Associativity::Left),
            Token::Equal(_) => Affix::Infix(Precedence(1), Associativity::Neither),
            Token::LParan(_) | Token::LBracket(_) => match &self.prev_token {
                Token::LCurly(_)
                | Token::RCurly(_)
                | Token::LParan(_)
                | Token::LBracket(_)
                | Token::Error
                | Token::Equal(_)
                | Token::Comma(_) => Affix::Nilfix,
                _ => Affix::Postfix(Precedence(10)),
            },
            Token::LCurly(_) => match &self.prev_token {
                Token::ClassIdentifier(_) => Affix::Postfix(Precedence(10)),
                _ => Affix::Nilfix,
            },
            Token::Arrow(_) => Affix::Nilfix,
            Token::RParan(_) => Affix::Nilfix,
            Token::Hash(_) => Affix::Nilfix,
            Token::RCurly(_) => Affix::Nilfix,
            Token::RBracket(_) => Affix::Nilfix,
            Token::NumberLiteral(_) => Affix::Nilfix,
            Token::StringLiteral(_) => Affix::Nilfix,
            Token::ClassIdentifier(_) => Affix::Nilfix,
            Token::BooleanLiteral(_) => Affix::Nilfix,
            Token::Identifier(_) => Affix::Nilfix,
            Token::Null(_) => Affix::Nilfix,
            _ => unreachable!("{:?}", token),
        };
        Ok(res)
    }
    fn primary(
        &mut self,
        token: Self::Input,
        rest: &mut Peekable<&mut I>,
    ) -> pratt::Result<Self::Output> {
        self.prev_token = token.clone();
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
                Expression::ClassIdentifier(ident)
            }
            Token::Identifier(data) => {
                let ident = data.value.to_string();
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

                if let Some(Token::Arrow(_)) = rest.peek() {
                    rest.next();
                    let arg_names = tokens
                        .iter()
                        .map(|t| t.text().to_string())
                        .collect::<Vec<String>>();
                    // If next token curly then treat it as code block, else parse expression
                    if let Some(Token::LCurly(_)) = rest.peek() {
                        rest.next();
                        let mut level = 0;
                        let mut body_tokens = Vec::new();
                        while let Some(token) = rest.next() {
                            match token {
                                Token::LCurly(_) => level += 1,
                                Token::RCurly(_) => {
                                    if level == 0 {
                                        break;
                                    } else {
                                        level -= 1;
                                    }
                                }
                                _ => body_tokens.push(token),
                            };
                        }
                        //TODO: This is really hacky
                        //      Try to refactor this somehow
                        let body = body_tokens
                            .iter()
                            .filter(|t| !t.is_whitespace())
                            .map(|t| t.text().to_string())
                            .join(" ");
                        let mut parser = Parser::new();
                        parser.lexer = itertools::multipeek(Lexer::new(&body));
                        return Ok(Expression::ArrowFunction(arg_names, parser.parse().stmts));
                    } else {
                        let mut tokens = Vec::new();
                        while let Some(token) = rest.next() {
                            tokens.push(token);
                        }
                        return Ok(Expression::ArrowFunction(
                            arg_names,
                            vec![Ast::ReturnStatement(
                                self.parse(&mut tokens.into_iter()).unwrap(),
                            )],
                        ));
                    }
                } else {
                    //TODO: handle error
                    return self
                        .parse(&mut tokens.into_iter())
                        .map_err(|_| pratt::NoError);
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
        lhs: Self::Output,
        token: Self::Input,
        rest: &mut Peekable<&mut I>,
    ) -> pratt::Result<Self::Output> {
        match token {
            Token::LBracket(_) => {
                let tokens = parse_inside_brackets(self, rest);
                Ok(Expression::PostOp(
                    Box::new(lhs),
                    "[]".into(),
                    Some(Box::new(self.parse(&mut tokens.into_iter()).unwrap())),
                ))
            }
            Token::LParan(_) => {
                let args = parse_inside_parenthesis(self, rest);
                Ok(Expression::PostOp(
                    Box::new(lhs),
                    "()".into(),
                    Some(Box::new(Expression::ArrayLiteral(args))),
                ))
            }
            Token::LCurly(_) => {
                let fields = parse_inside_curlies(self, rest);
                Ok(Expression::PostOp(
                    Box::new(lhs),
                    "{}".into(),
                    Some(Box::new(Expression::ObjectLiteral(fields))),
                ))
            }
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::ExprParser;
    use crate::{ast::Ast, expression::Expression::*};
    use crate::{expression::Expression, token::Token};
    use logos::Logos;
    use pratt::PrattParser;
    use std::collections::HashMap;

    fn parse(input: &str) -> Expression {
        ExprParser::default()
            .parse(&mut Token::lexer(input))
            .unwrap()
    }

    fn binary(lhs: Expression, op: &str, rhs: Expression) -> Expression {
        Expression::BinaryOp(Box::new(lhs), op.into(), Box::new(rhs))
    }

    fn post(lhs: Expression, op: &str, value: Option<Expression>) -> Expression {
        Expression::PostOp(Box::new(lhs), op.into(), value.map(|x| Box::new(x)))
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

    fn namespace_op(lhs: Expression, rhs: Expression) -> Expression {
        binary(lhs, "::", rhs)
    }

    fn number(x: i32) -> Expression {
        Expression::NumberLiteral(x)
    }

    fn boolean(x: bool) -> Expression {
        Expression::BooleanLiteral(x)
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
        assert_eq!(
            parse("1 + 2"),
            binary(NumberLiteral(1), "+", NumberLiteral(2),)
        );
    }

    #[test]
    fn math_expr2() {
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
    fn math_expr3() {
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
    fn math_expr4() {
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
    fn op_plus() {
        assert_eq!(
            parse(r#"a + " world""#),
            binary(ident("a"), "+", StringLiteral(" world".into()),)
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
            parse(r#"user::call()"#),
            call_op(namespace_op(ident("user"), ident("call")), vec![])
        );
    }

    #[test]
    fn nested_namespace_op() {
        assert_eq!(
            parse(r#"user::functions::call"#),
            namespace_op(
                namespace_op(ident("user"), ident("functions")),
                ident("call")
            )
        );
    }

    #[test]
    fn nested_namespace_op_with_fn_call() {
        assert_eq!(
            parse(r#"user::functions::call()"#),
            call_op(
                namespace_op(
                    namespace_op(ident("user"), ident("functions")),
                    ident("call")
                ),
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

    #[test]
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
    fn constructor_op() {
        assert_eq!(
            parse("User{}"),
            post(class("User"), "{}", Some(object(hashmap! {})))
        );
    }

    #[test]
    fn constructor_op_with_namespace() {
        assert_eq!(
            parse("lib::User{}"),
            post(
                binary(ident("lib"), "::", class("User")),
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
