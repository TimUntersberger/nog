use itertools::Itertools;
use std::collections::HashMap;

use super::{
    ast::Ast,
    operator::Operator,
    token::{Token, TokenKind},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    BinaryOp(Box<Expression>, Operator, Box<Expression>),
    PostOp(Box<Expression>, Operator, Option<Box<Expression>>),
    PreOp(Operator, Box<Expression>),
    NumberLiteral(String),
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
            match self {
                Self::Null => "null".into(),
                Self::Identifier(text)
                | Self::ClassIdentifier(text)
                | Self::StringLiteral(text)
                | Self::NumberLiteral(text)
                | Self::BooleanLiteral(text) => text.clone(),
                Self::ArrayLiteral(items) => format!(
                    "[{}]",
                    items.into_iter().map(|expr| expr.to_string()).join(", ")
                ),
                Self::ObjectLiteral(fields) => format!(
                    "#{{{}}}",
                    fields
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                        .join("\n")
                ),
                Self::ArrowFunction(args, _) =>
                    format!("({}) => {{ ... }}", args.into_iter().join(", ")),
                Self::ClassInstantiation(name, fields) => format!(
                    "{}{{{}}}",
                    name,
                    fields
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                        .join("\n")
                ),
                Self::PreOp(op, expr) => format!("{}{}", op.to_string(), expr.to_string()),
                Self::BinaryOp(lhs, op, rhs) => match op {
                    Operator::Dot =>
                        format!("{}{}{}", lhs.to_string(), op.to_string(), rhs.to_string()),
                    Operator::Call => format!("{}({})", lhs.to_string(), rhs.to_string()),
                    _ => format!("{} {} {}", lhs.to_string(), op.to_string(), rhs.to_string()),
                },
                Self::PostOp(lhs, op, _value) => format!("{}{}", lhs.to_string(), op.to_string()),
            }
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
