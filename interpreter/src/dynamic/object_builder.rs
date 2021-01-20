use std::{collections::HashMap, sync::Arc, sync::Mutex};

use crate::interpreter::Interpreter;
use crate::runtime_error::RuntimeResult;

use super::Dynamic;

pub struct ObjectBuilder {
    inner: HashMap<String, Dynamic>,
}

impl ObjectBuilder {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn function<T: Into<Dynamic>>(
        mut self,
        name: &str,
        f: impl Fn(&mut Interpreter, Vec<Dynamic>) -> RuntimeResult<T> + 'static + Send + Sync,
    ) -> Self {
        self.inner.insert(
            name.into(),
            Dynamic::RustFunction {
                name: name.into(),
                callback: Arc::new(move |a, b| f(a, b).map(|x| x.into())),
                scope: None,
            },
        );
        self
    }

    pub fn object(mut self, name: &str, obj: HashMap<String, Dynamic>) -> Self {
        self.inner
            .insert(name.into(), Dynamic::Object(Arc::new(Mutex::new(obj))));
        self
    }

    pub fn string(mut self, name: &str, value: &str) -> Self {
        self.inner
            .insert(name.into(), Dynamic::String(value.to_string()));
        self
    }

    pub fn build(self) -> HashMap<String, Dynamic> {
        self.inner
    }
}
