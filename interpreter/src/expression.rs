use itertools::Itertools;
use std::collections::HashMap;
use std::ops::Range;

use super::{
    ast::AstNode,
    formatter::Formatter,
    operator::Operator,
    token::{Token, TokenKind},
};

#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionKind {
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
    ArrowFunction(Vec<String>, Vec<AstNode>),
    ClassInstantiation(String, HashMap<String, Expression>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub location: Range<usize>,
}

impl Expression {
    pub fn new(kind: ExpressionKind, location: Range<usize>) -> Self {
        Self { kind, location }
    }
}

impl From<ExpressionKind> for Expression {
    fn from(value: ExpressionKind) -> Self {
        Self::new(value, 0..0)
    }
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

impl From<i32> for ExpressionKind {
    fn from(val: i32) -> Self {
        ExpressionKind::NumberLiteral(val.to_string())
    }
}

impl From<bool> for ExpressionKind {
    fn from(val: bool) -> Self {
        ExpressionKind::BooleanLiteral(val.to_string())
    }
}
