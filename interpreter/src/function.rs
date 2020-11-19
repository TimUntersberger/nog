use std::{fmt::Debug, sync::Arc};

use super::{dynamic::Dynamic, interpreter::Interpreter};

#[derive(Clone)]
pub struct Function {
    name: String,
    inner: Arc<dyn Fn(&mut Interpreter, Vec<Dynamic>) -> Dynamic>,
}

impl Function {
    pub fn invoke(&self, i: &mut Interpreter, args: Vec<Dynamic>) -> Dynamic {
        (self.inner)(i, args)
    }

    pub fn new<T>(name: &str, f: T) -> Self
    where
        T: Fn(&mut Interpreter, Vec<Dynamic>) -> Dynamic + 'static,
    {
        Self {
            name: name.to_string(),
            inner: Arc::new(f),
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "function {}(...)", self.name)
    }
}
