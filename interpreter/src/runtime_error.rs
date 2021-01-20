use super::dynamic::Dynamic;
use super::expression::Expression;
use super::interpreter::Program;
use super::operator::Operator;

#[derive(Clone, Debug)]
pub enum RuntimeError {
    StaticFunctionNotFound {
        class: String,
        function_name: String,
    },
    ClassNotFound {
        name: String,
    },
    ModuleNotFound {
        name: String,
    },
    Raw {
        msg: String,
    },
    UnexpectedType {
        expected: String,
        actual: String,
    },
    OperatorNotImplemented {
        expr: Expression,
        class: String,
        operator: Operator,
    },
}

impl RuntimeError {
    pub fn message(self, program: &Program) -> String {
        match self {
            RuntimeError::Raw { msg } => msg,
            RuntimeError::StaticFunctionNotFound {
                class,
                function_name,
            } => format!(
                "Class {} doesn't have a static function called {}",
                &class, &function_name
            ),
            RuntimeError::ClassNotFound { name } => {
                format!("Class {} couldn't be found in the current scope", &name)
            }
            RuntimeError::ModuleNotFound { name } => format!("Module {} couldn't be found", &name),
            RuntimeError::UnexpectedType { expected, actual } => {
                format!("Expected type {}, but found {}", &expected, &actual)
            }
            RuntimeError::OperatorNotImplemented {
                class,
                operator,
                expr,
            } => format!(
                "Class {} doesn't have operator {} implemented",
                &class,
                &operator.to_string(),
            ),
        }
    }
}

impl<T: Into<String>> From<T> for RuntimeError {
    fn from(val: T) -> Self {
        RuntimeError::Raw { msg: val.into() }
    }
}

pub type RuntimeResult<T = Dynamic> = Result<T, RuntimeError>;
