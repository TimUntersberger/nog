use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::{runtime_error::RuntimeResult, dynamic::Dynamic, interpreter::Interpreter};

#[derive(Default, Debug, Clone)]
pub struct Scope {
    pub variables: Arc<Mutex<HashMap<String, Dynamic>>>,
}

impl Scope {
    pub fn set(&mut self, key: String, value: Dynamic) {
        self.variables.lock().unwrap().insert(key, value);
    }

    pub fn get(&self, key: &str) -> Dynamic {
        self.variables
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    pub fn is_defined(&self, key: &str) -> bool {
        self.variables.lock().unwrap().contains_key(key)
    }

    pub fn register_rust_function(
        &mut self,
        name: &str,
        callback: impl Fn(&mut Interpreter, Vec<Dynamic>) -> RuntimeResult + 'static + Send + Sync,
    ) {
        self.set(
            name.to_string(),
            Dynamic::RustFunction {
                name: name.to_string(),
                callback: Arc::new(callback),
                scope: None,
            },
        )
    }
}

impl From<&Vec<Scope>> for Scope {
    fn from(scopes: &Vec<Scope>) -> Scope {
        let mut flat_scope = Scope::default();

        for scope in scopes {
            for (key, value) in scope.variables.lock().unwrap().iter() {
                flat_scope.set(key.clone(), value.clone());
            }
        }

        flat_scope
    }
}
