use itertools::Itertools;
use std::{
    any::Any,
    collections::HashMap,
    fmt::{Debug, Display},
    sync::Arc,
    sync::Mutex,
};

use super::{
    ast::AstNode,
    class::Class,
    expression::Expression,
    function::Function,
    interpreter::Interpreter,
    module::Module,
    runtime_error::{RuntimeError, RuntimeResult},
    scope::Scope,
};

pub mod object_builder;

pub type Number = i32;

#[derive(Clone)]
pub enum Dynamic {
    String(String),
    Number(Number),
    RustValue(Arc<Box<dyn Any + Send + Sync>>),
    Boolean(bool),
    Lazy(Expression),
    Array(Arc<Mutex<Vec<Dynamic>>>),
    Object(Arc<Mutex<HashMap<String, Dynamic>>>),
    Module(Module),
    Class(Class),
    Function {
        name: String,
        arg_names: Vec<String>,
        body: Vec<AstNode>,
        scope: Scope,
    },
    RustFunction {
        name: String,
        callback: Arc<dyn Fn(&mut Interpreter, Vec<Dynamic>) -> RuntimeResult + Send + Sync>,
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
            Dynamic::ClassInstance(name, fields_ref) => {
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
                let mut fields = fields.lock().unwrap();
                if fields.contains_key(key) {
                    fields.insert(key.to_string(), value)
                } else {
                    None
                }
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

    pub fn as_array(self) -> RuntimeResult<Vec<Dynamic>> {
        match self {
            Dynamic::Array(items) => Ok(items.lock().unwrap().clone()),
            x => Err(RuntimeError::UnexpectedType {
                expected: "Array".into(),
                actual: x.type_name(),
            }),
        }
    }

    pub fn as_fn(self) -> RuntimeResult<Function> {
        match self {
            Dynamic::Function {
                name,
                scope,
                body,
                arg_names,
            } => Ok(Function::new(&name, Some(scope.clone()), move |i, args| {
                let body = body.clone();
                let arg_names = arg_names.clone();
                let scope = scope.clone();
                i.call_fn(None, Some(scope), &arg_names, &args, &body)
            })),
            Dynamic::RustFunction {
                name,
                scope,
                callback,
            } => {
                let callback = callback.clone();

                Ok(Function::new(&name, scope.clone(), move |i, args| {
                    let args = args.clone();
                    callback(i, args)
                }))
            }
            x => Err(RuntimeError::UnexpectedType {
                expected: "Function".into(),
                actual: x.type_name(),
            }),
        }
    }

    pub fn as_str(self) -> RuntimeResult<String> {
        match self {
            Dynamic::String(string) => Ok(string),
            x => Err(RuntimeError::UnexpectedType {
                expected: "Function".into(),
                actual: x.type_name(),
            }),
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            Dynamic::String(_) => "String",
            Dynamic::Number(_) => "Number",
            Dynamic::RustValue(_) => "RustValue",
            Dynamic::Lazy(_) => "Lazy",
            Dynamic::Module(_) => "Module",
            Dynamic::Boolean(_) => "Boolean",
            Dynamic::Class(_) => "Class",
            Dynamic::Array(_) => "Array",
            Dynamic::Object(_) => "Object",
            Dynamic::ClassInstance(name, _) => name,
            Dynamic::Function { .. } => "Function",
            Dynamic::RustFunction { .. } => "RustFunction",
            Dynamic::Null => "Null",
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
            Dynamic::Null => match other {
                Dynamic::Null => true,
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

impl From<char> for Dynamic {
    fn from(val: char) -> Self {
        Dynamic::String(val.into())
    }
}

impl From<&str> for Dynamic {
    fn from(val: &str) -> Self {
        Dynamic::String(val.to_string())
    }
}

impl From<usize> for Dynamic {
    fn from(val: usize) -> Self {
        Dynamic::Number(val as Number)
    }
}

impl From<i32> for Dynamic {
    fn from(val: i32) -> Self {
        Dynamic::Number(val as Number)
    }
}

impl From<i64> for Dynamic {
    fn from(val: i64) -> Self {
        Dynamic::Number(val as Number)
    }
}

impl From<()> for Dynamic {
    fn from(_: ()) -> Self {
        Dynamic::Null
    }
}

impl From<Arc<Box<dyn Any + Send + Sync>>> for Dynamic {
    fn from(ptr: Arc<Box<dyn Any + Send + Sync>>) -> Self {
        Dynamic::RustValue(ptr.clone())
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
            Dynamic::String(string) => string.clone(),
            Dynamic::RustValue(expr) => todo!(),
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
                    "#{}".to_string()
                } else {
                    format!(
                        "#{{\n{}\n}}",
                        fields
                            .iter()
                            .map(|(k, v)| indent(format!("\"{}\": {}", k, v)))
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
