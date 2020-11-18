use std::collections::HashMap;

use super::{class::Class, dynamic::Dynamic, function::Function, scope::Scope};

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub scope: Scope,
    pub variables: HashMap<String, Dynamic>,
    pub functions: HashMap<String, Function>,
    pub classes: HashMap<String, Class>,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            scope: Scope::default(),
            variables: HashMap::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
        }
    }
}
