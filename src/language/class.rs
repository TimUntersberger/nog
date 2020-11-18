use std::{collections::HashMap, fmt::Debug};

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
        .set_op_impl(Operator::Dot, |interp, this, args| {
            let class_name = this.type_name();
            let fields_ref = match &this {
                Dynamic::ClassInstance(_, fields) => fields.clone(),
                Dynamic::Object(fields) => fields.clone(),
                _ => Default::default(),
            };
            let fields = fields_ref.lock().unwrap();

            let rhs = &args[0];

            match rhs {
                Dynamic::String(x) => fields.get(x).unwrap_or_default().clone(),
                Dynamic::Array(vec_ref) => {
                    let vec = vec_ref.lock().unwrap();
                    let fn_name = vec[0].clone().as_str().unwrap();
                    let args = vec[1].clone().as_array().unwrap();

                    if class_name == "object" {
                        if let Some(var) = fields.get(&fn_name) {
                            match var {
                                Dynamic::RustFunction { callback, .. } => {
                                    callback(interp, args).unwrap_or_default()
                                },
                                _ => todo!()
                            }
                        } else {
                            todo!("{}", fn_name)
                        }
                    } else {
                        let class = interp.classes.get(&class_name).unwrap().clone();

                        if let Some(method) = class.functions.get(&fn_name) {
                            method.invoke(interp, this, args)
                        } else {
                            todo!("{}", fn_name)
                        }
                    }
                }
                _ => todo!(),
            }
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
            .insert(name.to_string(), Function::new(name, f));
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
        let fields = HashMap::new();
        Dynamic::new_object(fields)
    }
}
