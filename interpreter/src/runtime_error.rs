use super::dynamic::Dynamic;
use super::operator::Operator;

#[derive(Clone, Debug)]
pub enum RuntimeError {
    ClassNotFound { name: String },
    UnexpectedType { expected: String, actual: String },
    OperatorNotImplemented { class: String, operator: Operator }
}

impl RuntimeError {
    pub fn message(self) -> String {
        match self {
            RuntimeError::ClassNotFound { name } => {
                format!("Class {} couldn't be found in the current scope", &name)
            }
            RuntimeError::UnexpectedType { expected, actual } => {
                format!("Expected type {}, but found {}", &expected, &actual)
            }
            RuntimeError::OperatorNotImplemented { class, operator } => {
                format!("Class {} doesn't have operator {} implemented", &class, &operator.to_string())
            }
        }
    }
}

pub type RuntimeResult<T = Dynamic> = Result<T, RuntimeError>;
