use itertools::Itertools;
use std::collections::HashMap;

use super::{
    ast::Ast,
    formatter::Formatter,
    operator::Operator,
    token::{Token, TokenKind},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    BinaryOp(Box<Expression>, Operator, Box<Expression>),
    PostOp(Box<Expression>, Operator, Option<Box<Expression>>),
    PreOp(Operator, Box<Expression>),
    NumberLiteral(String),
    HexLiteral(String),
    ArrayLiteral(Vec<Expression>),
    ObjectLiteral(HashMap<String, Expression>),
    BooleanLiteral(String),
    StringLiteral(String),
    Identifier(String),
    ClassIdentifier(String),
    Null,
    ArrowFunction(Vec<String>, Vec<Ast>),
    ClassInstantiation(String, HashMap<String, Expression>),
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            Formatter::new(&Default::default()).format_expr(self)
        )
    }
}

impl From<i32> for Expression {
    fn from(val: i32) -> Self {
        Expression::NumberLiteral(val.to_string())
    }
}

impl From<bool> for Expression {
    fn from(val: bool) -> Self {
        Expression::BooleanLiteral(val.to_string())
    }
}
