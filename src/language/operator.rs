#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Operator {
    Add,
    Dot,
    Assign,
    Namespace,
    Pipe,
}

impl Operator {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "+" => Operator::Add,
            "." => Operator::Dot,
            "=" => Operator::Assign,
            "::" => Operator::Namespace,
            "|>" => Operator::Pipe,
            _ => return None,
        })
    }

    pub fn method_name(&self) -> String {
        match self {
            Operator::Add => "add",
            Operator::Dot => "dot",
            Operator::Assign => "set",
            Operator::Namespace => "namespace",
            Operator::Pipe => "pipe",
        }
        .into()
    }

    pub fn to_string(&self) -> String {
        match self {
            Operator::Add => "+",
            Operator::Dot => ".",
            Operator::Assign => "=",
            Operator::Namespace => "::",
            Operator::Pipe => "|>",
        }
        .into()
    }
}
