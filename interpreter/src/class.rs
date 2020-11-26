use std::{collections::HashMap, fmt::Debug, sync::Arc};

use super::{
    dynamic::Dynamic, expression::Expression, function::Function, interpreter::Interpreter,
    method::Method, operator::Operator,
};

#[derive(Clone, Debug)]
pub struct Class {
    pub name: String,
    pub fields: HashMap<String, Expression>,
    pub static_functions: HashMap<String, Function>,
    pub functions: HashMap<String, Method>,
    pub op_impls: HashMap<Operator, Method>,
}

impl Class {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: HashMap::new(),
            static_functions: HashMap::new(),
            functions: HashMap::new(),
            op_impls: HashMap::new(),
        }
        .set_op_impl(Operator::Assign, |_, this, args| {
            let key = args[0].clone().as_str().unwrap();
            let value = &args[1];
            let fields_ref = match &this {
                Dynamic::ClassInstance(_, fields) => fields.clone(),
                Dynamic::Object(fields) => fields.clone(),
                _ => Default::default(),
            };
            let mut fields = fields_ref.lock().unwrap();
            fields.insert(key.clone(), value.clone());
            value.clone()
        })
        .set_op_impl(Operator::Dot, |_, this, args| {
            let field = args[0].clone().as_str().unwrap();
            this.get_field(&field)
        })
        .set_op_impl(Operator::And, |_, this, args| {
            let lhs = this.is_true();
            let rhs = args[0].is_true();
            Dynamic::Boolean(lhs && rhs)
        })
        .set_op_impl(Operator::Or, |_, this, args| {
            let lhs = this.is_true();
            let rhs = args[0].is_true();
            Dynamic::Boolean(lhs || rhs)
        })
    }

    pub fn add_field(mut self, name: &str, default: Expression) -> Self {
        self.fields.insert(name.to_string(), default);
        self
    }

    pub fn add_function(
        mut self,
        name: &str,
        f: impl Fn(&mut Interpreter, Dynamic, Vec<Dynamic>) -> Dynamic + 'static,
    ) -> Self {
        self.functions
            .insert(name.to_string(), Method::new(name, f));
        self
    }

    pub fn add_static_function(
        mut self,
        name: &str,
        f: impl Fn(&mut Interpreter, Vec<Dynamic>) -> Dynamic + 'static,
    ) -> Self {
        self.static_functions
            .insert(name.to_string(), Function::new(name, None, f));
        self
    }

    pub fn set_op_impl(
        mut self,
        op: Operator,
        f: impl Fn(&mut Interpreter, Dynamic, Vec<Dynamic>) -> Dynamic + 'static,
    ) -> Self {
        let op_method_name = op.method_name();
        self.op_impls.insert(op, Method::new(&op_method_name, f));
        self
    }

    pub fn get_op_impl(&self, op: &Operator) -> Option<&Method> {
        self.op_impls.get(op)
    }
}

impl Into<Dynamic> for Class {
    fn into(self) -> Dynamic {
        Dynamic::Class(self)
    }
}
