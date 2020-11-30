use super::token::TokenKind;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Operator {
    Add,
    Subtract,
    Divide,
    Times,
    Dot,
    Assign,
    Increment,
    Decrement,
    Constructor,
    Pipe,
    Index,
    Call,
    GreaterThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    LessThan,
    Equal,
    NotEqual,
    Not,
    And,
    Or,
}

impl Operator {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "+" => Operator::Add,
            "-" => Operator::Subtract,
            "/" => Operator::Divide,
            "*" => Operator::Times,
            "." => Operator::Dot,
            "=" => Operator::Assign,
            "++" => Operator::Increment,
            "--" => Operator::Decrement,
            "|>" => Operator::Pipe,
            "[]" => Operator::Index,
            "()" => Operator::Call,
            "{}" => Operator::Constructor,
            ">" => Operator::GreaterThan,
            ">=" => Operator::GreaterThanOrEqual,
            "<" => Operator::LessThan,
            "<=" => Operator::LessThanOrEqual,
            "==" => Operator::Equal,
            "!=" => Operator::NotEqual,
            "!" => Operator::Not,
            "&&" => Operator::And,
            "||" => Operator::Or,
            _ => return None,
        })
    }

    pub fn method_name(&self) -> String {
        match self {
            Operator::Add => "add",
            Operator::Subtract => "subtract",
            Operator::Times => "multiply",
            Operator::Divide => "divide",
            Operator::Dot => "dot",
            Operator::Assign => "set",
            Operator::Pipe => "pipe",
            Operator::Increment => "increment",
            Operator::Decrement => "decrement",
            Operator::Constructor => "{}",
            Operator::Index => "index",
            Operator::Call => "call",
            Operator::GreaterThan => "greater_than",
            Operator::GreaterThanOrEqual => "greater_than_or_equal",
            Operator::LessThan => "less_than",
            Operator::LessThanOrEqual => "less_than_or_equal",
            Operator::NotEqual => "not_equal",
            Operator::Equal => "equal",
            Operator::Not => "not",
            Operator::And => "and",
            Operator::Or => "or",
        }
        .into()
    }

    pub fn to_string(&self) -> String {
        match self {
            Operator::Add => "+",
            Operator::Subtract => "-",
            Operator::Times => "*",
            Operator::Divide => "/",
            Operator::Dot => ".",
            Operator::Assign => "=",
            Operator::Pipe => "|>",
            Operator::Index => "[]",
            Operator::Call => "()",
            Operator::Constructor => "{}",
            Operator::GreaterThan => ">",
            Operator::GreaterThanOrEqual => ">=",
            Operator::LessThan => "<",
            Operator::Increment => "++",
            Operator::Decrement => "--",
            Operator::LessThanOrEqual => "<=",
            Operator::NotEqual => "!=",
            Operator::Equal => "==",
            Operator::Not => "!",
            Operator::And => "&&",
            Operator::Or => "||",
        }
        .into()
    }
}

impl From<TokenKind> for Operator {
    fn from(value: TokenKind) -> Self {
        match value {
            TokenKind::Plus => Operator::Add,
            TokenKind::PlusPlus => Operator::Increment,
            TokenKind::Minus => Operator::Subtract,
            TokenKind::MinusMinus => Operator::Decrement,
            TokenKind::Star => Operator::Times,
            TokenKind::Slash => Operator::Divide,
            TokenKind::Dot => Operator::Dot,
            TokenKind::Equal => Operator::Assign,
            TokenKind::ExclamationMark => Operator::Not,
            TokenKind::NEQ => Operator::NotEqual,
            TokenKind::EQ => Operator::Equal,
            TokenKind::And => Operator::And,
            TokenKind::Or => Operator::Or,
            TokenKind::GT => Operator::GreaterThan,
            TokenKind::LT => Operator::LessThan,
            TokenKind::GTE => Operator::GreaterThanOrEqual,
            TokenKind::LTE => Operator::LessThanOrEqual,
            _ => todo!("{:?}", value),
        }
    }
}
// "+" => Operator::Add,
//           "-" => Operator::Subtract,
//           "/" => Operator::Divide,
//           "*" => Operator::Times,
//           "." => Operator::Dot,
//           "=" => Operator::Assign,
//           "++" => Operator::Increment,
//           "--" => Operator::Decrement,
//           "|>" => Operator::Pipe,
//           "[]" => Operator::Index,
//           "()" => Operator::Call,
//           "{}" => Operator::Constructor,
//           ">" => Operator::GreaterThan,
//           ">=" => Operator::GreaterThanOrEqual,
//           "<" => Operator::LessThan,
//           "<=" => Operator::LessThanOrEqual,
//           "==" => Operator::Equal,
//           "!=" => Operator::NotEqual,
//           "!" => Operator::Not,
//           "&&" => Operator::And,
//           "||" => Operator::Or,
