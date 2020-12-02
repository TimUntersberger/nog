use itertools::Itertools;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::Arc,
    sync::Mutex,
};

use super::{
    ast::Ast, class::Class, expression::Expression, interpreter::Interpreter, module::Module,
    scope::Scope,
};

pub mod object_builder;

#[derive(Clone)]
pub enum Dynamic {
    String(String),
    Number(i32),
    Boolean(bool),
    Lazy(Expression),
    Array(Arc<Mutex<Vec<Dynamic>>>),
    Object(Arc<Mutex<HashMap<String, Dynamic>>>),
    Module(Module),
    Class(Class),
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
            Dynamic::Module(module) => module
                .variables
                .get(key)
                .cloned()
                .or_else(|| module.functions.get(key).map(|x| x.clone().into()))
                .or_else(|| module.classes.get(key).map(|x| x.clone().into()))
                .unwrap_or_default(),
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
            Dynamic::Module(_) => "module",
            Dynamic::Boolean(_) => "boolean",
            Dynamic::Class(_) => "class",
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

impl std::ops::Add for Dynamic {
    type Output = Dynamic;

    fn add(self, other: Dynamic) -> Self::Output {
        match self {
            Dynamic::Number(x) => match other {
                Dynamic::Number(y) => (x + y).into(),
                _ => Dynamic::Null,
            },
            Dynamic::String(x) => match other {
                Dynamic::String(y) => format!("{}{}", x, y).into(),
                Dynamic::Boolean(y) => format!("{}{}", x, y).into(),
                Dynamic::Number(y) => format!("{}{}", x, y).into(),
                _ => Dynamic::Null,
            },
            Dynamic::Array(x) => match other {
                Dynamic::Array(y) => Dynamic::new_array(
                    x.lock()
                        .unwrap()
                        .clone()
                        .into_iter()
                        .chain(y.lock().unwrap().clone())
                        .collect_vec(),
                ),
                _ => Dynamic::Null,
            },
            _ => Dynamic::Null,
        }
        .into()
    }
}

impl std::ops::Sub for Dynamic {
    type Output = Dynamic;

    fn sub(self, other: Dynamic) -> Self::Output {
        match self {
            Dynamic::Number(x) => match other {
                Dynamic::Number(y) => (x - y).into(),
                _ => Dynamic::Null,
            },
            _ => Dynamic::Null,
        }
        .into()
    }
}

impl std::ops::Mul for Dynamic {
    type Output = Dynamic;

    fn mul(self, other: Dynamic) -> Self::Output {
        match self {
            Dynamic::Number(x) => match other {
                Dynamic::Number(y) => (x * y).into(),
                _ => Dynamic::Null,
            },
            _ => Dynamic::Null,
        }
        .into()
    }
}

impl std::ops::Div for Dynamic {
    type Output = Dynamic;

    fn div(self, other: Dynamic) -> Self::Output {
        match self {
            Dynamic::Number(x) => match other {
                Dynamic::Number(y) => (x / y).into(),
                _ => Dynamic::Null,
            },
            _ => Dynamic::Null,
        }
        .into()
    }
}

impl std::cmp::PartialEq for Dynamic {
    fn eq(&self, other: &Dynamic) -> bool {
        match self {
            Dynamic::Number(x) => match other {
                Dynamic::Number(y) => x == y,
                _ => false,
            },
            Dynamic::String(x) => match other {
                Dynamic::String(y) => x == y,
                _ => false,
            },
            Dynamic::Boolean(x) => match other {
                Dynamic::Boolean(y) => x == y,
                _ => false,
            },
            _ => false,
        }
    }
}

impl std::cmp::PartialOrd for Dynamic {
    fn partial_cmp(&self, other: &Dynamic) -> Option<std::cmp::Ordering> {
        match self {
            Dynamic::Number(x) => match other {
                Dynamic::Number(y) => Some(x.cmp(y)),
                _ => None,
            },
            Dynamic::String(x) => match other {
                Dynamic::String(y) => Some(x.cmp(y)),
                _ => None,
            },
            _ => None,
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

impl From<Module> for Dynamic {
    fn from(val: Module) -> Self {
        Dynamic::Module(val)
    }
}

impl From<String> for Dynamic {
    fn from(val: String) -> Self {
        Dynamic::String(val)
    }
}

impl From<&str> for Dynamic {
    fn from(val: &str) -> Self {
        Dynamic::String(val.to_string())
    }
}

impl From<i32> for Dynamic {
    fn from(val: i32) -> Self {
        Dynamic::Number(val)
    }
}

impl From<()> for Dynamic {
    fn from(_: ()) -> Self {
        Dynamic::Null
    }
}

impl From<bool> for Dynamic {
    fn from(val: bool) -> Self {
        Dynamic::Boolean(val)
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
            Dynamic::Lazy(expr) => todo!(),
            Dynamic::Module(module) => format!("module {:#?}", module),
            Dynamic::Class(class) => format!("class {}", class.name),
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