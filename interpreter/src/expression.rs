use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

use super::ast::Ast;

/// TODO: replace String with &str
#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    BinaryOp(Box<Expression>, String, Box<Expression>),
    NumberLiteral(i32),
    ArrayLiteral(Vec<Expression>),
    ObjectLiteral(HashMap<String, Expression>),
    BooleanLiteral(bool),
    StringLiteral(String),
    Identifier(String),
    ClassIdentifier(String),
    Null,
    ArrowFunction(Vec<String>, Vec<Ast>),
    ClassInstantiation(String, HashMap<String, Expression>),
    FunctionCall(Box<Expression>, Vec<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Expression::Identifier(x) => x.clone(),
                Expression::ClassIdentifier(x) => x.clone(),
                Expression::StringLiteral(x) => format!("\"{}\"", x.clone()),
                Expression::NumberLiteral(x) => x.to_string(),
                Expression::ArrowFunction(arg_names, _) => format!(
                    "({}) => {{ ... }}",
                    arg_names.iter().map(|a| a.to_string()).join(", ")
                ),
                Expression::BooleanLiteral(x) => x.to_string(),
                //TODO: might have to modify the to string for function call expressions
                Expression::FunctionCall(name, args) => format!(
                    "{}({})",
                    name,
                    args.iter().map(|a| a.to_string()).join(", ")
                ),
                Expression::BinaryOp(lhs, op, rhs) => match op.as_str() {
                    "." => format!("{}{}{}", lhs.to_string(), op, rhs.to_string()),
                    _ => format!("{} {} {}", lhs.to_string(), op, rhs.to_string()),
                },
                _ => "unknown".into(),
            }
        )
    }
}
