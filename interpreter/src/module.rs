use std::collections::HashMap;

use super::{
    class::Class, dynamic::Dynamic, function::Function, interpreter::Interpreter, scope::Scope,
};

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

    pub fn variable<T: Into<Dynamic>>(mut self, name: &str, value: T) -> Self {
        let value = value.into();
        self.variables.insert(name.to_string(), value.clone());
        self.scope.set(name.to_string(), value);
        self
    }

    pub fn function<
        R: Into<Dynamic>,
        T: Fn(&mut Interpreter, Vec<Dynamic>) -> R + 'static + Send,
    >(
        mut self,
        name: &str,
        value: T,
    ) -> Self {
        let value: Dynamic = Function::new(name, Some(self.scope.clone()), move |i, args| {
            value(i, args).into()
        })
        .into();
        self.variables.insert(name.to_string(), value.clone());
        self.scope.set(name.to_string(), value);
        self
    }
}
