use itertools::Itertools;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::Arc,
    sync::Mutex,
};

use super::{ast::Ast, expression::Expression, interpreter::Interpreter, scope::Scope};

pub mod object_builder;

#[derive(Clone)]
pub enum Dynamic {
    String(String),
    Number(i32),
    Boolean(bool),
    Lazy(Expression),
    Array(Arc<Mutex<Vec<Dynamic>>>),
    Object(Arc<Mutex<HashMap<String, Dynamic>>>),
    Function {
        name: String,
        arg_names: Vec<String>,
        body: Vec<Ast>,
        scope: Scope,
    },
    RustFunction {
        name: String,
        callback: Arc<dyn Fn(&mut Interpreter, Vec<Dynamic>) -> Option<Dynamic>>,
        scope: Option<Scope>,
    },
    ClassInstance(String, Arc<Mutex<HashMap<String, Dynamic>>>),
    Null,
}

impl Dynamic {
    pub fn is_null(&self) -> bool {
        match self {
            Dynamic::Null => true,
            _ => false,
        }
    }

    /// Returns a copy of the field with the given name.
    /// If this function returns `null` it could either be because the field doesn't exist or it is
    /// set to null
    pub fn get_field(&self, key: &str) -> Dynamic {
        match self {
            Dynamic::Object(fields_ref) => {
                let fields = fields_ref.lock().unwrap();
                fields.get(key).cloned().unwrap_or_default()
            }
            Dynamic::ClassInstance(_, fields_ref) => {
                let fields = fields_ref.lock().unwrap();
                fields.get(key).cloned().unwrap_or_default()
            }
            _ => Dynamic::Null,
        }
    }

    /// Sets the field with the given name to the new value.
    /// This function returns the previous value of the field or `None` if the field doesn't exist.
    pub fn set_field(&self, key: &str, value: Dynamic) -> Option<Dynamic> {
        match self {
            Dynamic::Object(fields_ref) => {
                let mut fields = fields_ref.lock().unwrap();
                if fields.contains_key(key) {
                    fields.insert(key.to_string(), value)
                } else {
                    None
                }
            }
            Dynamic::ClassInstance(_, fields) => {
                todo!();
            }
            _ => None,
        }
    }

    pub fn new_array(items: Vec<Dynamic>) -> Self {
        Dynamic::Array(Arc::new(Mutex::new(items)))
    }

    pub fn new_object(fields: HashMap<String, Dynamic>) -> Self {
        Dynamic::Object(Arc::new(Mutex::new(fields)))
    }

    pub fn new_instance(name: &str, fields: HashMap<String, Dynamic>) -> Self {
        Dynamic::ClassInstance(name.to_string(), Arc::new(Mutex::new(fields)))
    }

    pub fn as_array(self) -> Option<Vec<Dynamic>> {
        match self {
            Dynamic::Array(items) => Some(items.lock().unwrap().clone()),
            _ => None,
        }
    }

    pub fn as_str(self) -> Option<String> {
        match self {
            Dynamic::String(string) => Some(string),
            _ => None,
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            Dynamic::String(_) => "string",
            Dynamic::Number(_) => "number",
            Dynamic::Lazy(_) => "lazy",
            Dynamic::Boolean(_) => "boolean",
            Dynamic::Array(_) => "array",
            Dynamic::Object(_) => "object",
            Dynamic::ClassInstance(name, _) => name,
            Dynamic::Function { .. } => "function",
            Dynamic::RustFunction { .. } => "extern function",
            Dynamic::Null => "null",
        }
        .into()
    }

    pub fn is_true(&self) -> bool {
        match self {
            Dynamic::Boolean(x) => *x,
            Dynamic::Null => false,
            _ => true,
        }
    }
}

impl From<Expression> for Dynamic {
    fn from(expr: Expression) -> Self {
        Dynamic::Lazy(expr)
    }
}

impl From<&Expression> for Dynamic {
    fn from(expr: &Expression) -> Self {
        Dynamic::Lazy(expr.clone())
    }
}

impl<T: Into<Dynamic>> Into<Dynamic> for HashMap<String, T> {
    fn into(self) -> Dynamic {
        Dynamic::Object(Arc::new(Mutex::new(
            self.into_iter().map(|(k, v)| (k, v.into())).collect(),
        )))
    }
}

impl<T: Into<Dynamic>> Into<Dynamic> for Vec<T> {
    fn into(self) -> Dynamic {
        Dynamic::Array(Arc::new(Mutex::new(
            self.into_iter().map(|x| x.into()).collect(),
        )))
    }
}

impl From<String> for Dynamic {
    fn from(val: String) -> Self {
        Dynamic::String(val)
    }
}

impl Debug for Dynamic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Dynamic::Function {
                name, arg_names, ..
            } => format!("Function({}, [{}])", name, arg_names.join(", ")),
            x => format!("{}", x), // if i use debug formatting it overflows the stack for some reason
        })
    }
}

impl Default for Dynamic {
    fn default() -> Self {
        Self::Null
    }
}

impl Default for &Dynamic {
    fn default() -> Self {
        &Dynamic::Null
    }
}

fn indent(lines: String) -> String {
    lines
        .to_string()
        .split("\n")
        .map(|line| format!("  {}", line))
        .join("\n")
}

impl Display for Dynamic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Dynamic::Boolean(boolean) => boolean.to_string(),
            Dynamic::String(string) => format!(r#""{}""#, string),
            Dynamic::Lazy(expr) => expr.to_string(),
            Dynamic::Array(items_ref) => {
                let items = items_ref.lock().unwrap();
                if items.is_empty() {
                    "[]".to_string()
                } else {
                    format!(
                        "[\n{}\n]",
                        items.iter().map(|i| indent(i.to_string())).join(",\n")
                    )
                }
            }
            Dynamic::Object(fields_ref) => {
                let fields = fields_ref.lock().unwrap();
                if fields.is_empty() {
                    "{}".to_string()
                } else {
                    format!(
                        "{{\n{}\n}}",
                        fields
                            .iter()
                            .map(|(k, v)| indent(format!("{}: {}", k, v)))
                            .join("\n")
                    )
                }
            }
            Dynamic::Number(number) => number.to_string(),
            Dynamic::ClassInstance(name, fields_ref) => {
                let fields = fields_ref.lock().unwrap();
                if fields.is_empty() {
                    format!("{} {{}}", name)
                } else {
                    format!(
                        "{} {{\n{}\n}}",
                        name,
                        fields
                            .iter()
                            .map(|(k, v)| indent(format!("{}: {}", k, v)))
                            .join("\n")
                    )
                }
            }
            Dynamic::Null => "null".into(),
            Dynamic::RustFunction { name, .. } => format!("extern function {}(...)", name),
            Dynamic::Function {
                name, arg_names, ..
            } => format!("function {}({})", name, arg_names.join(", ")),
        })
    }
}
