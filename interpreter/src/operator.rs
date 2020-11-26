#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Operator {
    Add,
    Dot,
    Assign,
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
            "." => Operator::Dot,
            "=" => Operator::Assign,
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
            Operator::Dot => "dot",
            Operator::Assign => "set",
            Operator::Pipe => "pipe",
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
            Operator::Dot => ".",
            Operator::Assign => "=",
            Operator::Pipe => "|>",
            Operator::Index => "[]",
            Operator::Call => "()",
            Operator::Constructor => "{}",
            Operator::GreaterThan => ">",
            Operator::GreaterThanOrEqual => ">=",
            Operator::LessThan => "<",
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
