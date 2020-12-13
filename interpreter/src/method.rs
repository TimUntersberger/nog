use std::{fmt::Debug, sync::Arc};

use super::{dynamic::Dynamic, function::Function, interpreter::Interpreter, runtime_error::RuntimeResult};

#[derive(Clone)]
pub struct Method {
    name: String,
    inner: Arc<dyn Fn(&mut Interpreter, Dynamic, Vec<Dynamic>) -> RuntimeResult + Send + Sync>,
}

impl Method {
    pub fn invoke(&self, i: &mut Interpreter, this: Dynamic, args: Vec<Dynamic>) -> RuntimeResult {
        (self.inner)(i, this, args)
    }

    pub fn new<T>(name: &str, f: T) -> Self
    where
        T: Fn(&mut Interpreter, Dynamic, Vec<Dynamic>) -> RuntimeResult + 'static + Send + Sync,
    {
        Method {
            name: name.to_string(),
            inner: Arc::new(f),
        }
    }

    pub fn into_dynamic(&self, this: Dynamic) -> Dynamic {
        let f = self.inner.clone();
        Dynamic::RustFunction {
            name: self.name.clone(),
            scope: None,
            callback: Arc::new(move |i, args| (f)(i, this.clone(), args)),
        }
    }

    pub fn into_fn(&self, this: Dynamic) -> Function {
        let f = self.inner.clone();
        Function::new(&self.name, None, move |i, args| f(i, this.clone(), args))
    }
}

impl Debug for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "method {}(...)", self.name)
    }
}
