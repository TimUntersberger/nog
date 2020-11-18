use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use super::{dynamic::Dynamic, interpreter::Interpreter};

#[derive(Default, Debug, Clone)]
pub struct Scope {
    pub variables: Rc<RefCell<HashMap<String, Dynamic>>>,
}

impl Scope {
    pub fn set(&mut self, key: String, value: Dynamic) {
        self.variables.borrow_mut().insert(key, value);
    }

    pub fn get(&self, key: &str) -> Dynamic {
        self.variables
            .borrow()
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    pub fn is_defined(&self, key: &str) -> bool {
        self.variables.borrow().contains_key(key)
    }

    pub fn register_rust_function(
        &mut self,
        name: &str,
        callback: impl Fn(&mut Interpreter, Vec<Dynamic>) -> Option<Dynamic> + 'static,
    ) {
        self.set(
            name.to_string(),
            Dynamic::RustFunction {
                name: name.to_string(),
                callback: Arc::new(callback),
            },
        )
    }
}
