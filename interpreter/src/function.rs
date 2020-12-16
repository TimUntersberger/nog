use std::{fmt::Debug, sync::Arc};

use super::{
    dynamic::Dynamic, interpreter::Interpreter, runtime_error::RuntimeResult, scope::Scope,
};

#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub scope: Scope,
    pub inner: Arc<dyn Fn(&mut Interpreter, Vec<Dynamic>) -> RuntimeResult + Send + Sync>,
}

impl Function {
    pub fn invoke(&self, i: &mut Interpreter, args: Vec<Dynamic>) -> RuntimeResult {
        i.scopes.push(self.scope.clone());
        let res = (self.inner)(i, args);
        i.scopes.pop();
        res
    }

    pub fn new<T>(name: &str, scope: Option<Scope>, f: T) -> Self
    where
        T: Fn(&mut Interpreter, Vec<Dynamic>) -> RuntimeResult + 'static + Send + Sync,
    {
        Self {
            name: name.to_string(),
            scope: scope.unwrap_or_default(),
            inner: Arc::new(f),
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "function {}(...)", self.name)
    }
}

impl Into<Dynamic> for Function {
    fn into(self) -> Dynamic {
        Dynamic::RustFunction {
            name: self.name.clone(),
            scope: Some(self.scope.clone()),
            callback: Arc::new(move |i, arg| self.invoke(i, arg)),
        }
    }
}
