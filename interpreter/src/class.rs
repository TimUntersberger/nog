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
        .set_op_impl(Operator::Dot, |i, this, args| {
            let field = args[0].clone().as_str().unwrap();
            let class_name = this.type_name();

            if class_name != "module" {
                if let Some(class) = i.find_class(&class_name) {
                    if let Some(f) = class.functions.get(&field) {
                        return f.into_dynamic(this);
                    }
                }
            }

            this.get_field(&field)
        })
        .set_op_impl(Operator::And, |_, this, args| {
            let lhs = this.is_true();
            let rhs = args[0].is_true();
            lhs && rhs
        })
        .set_op_impl(Operator::Or, |_, this, args| {
            let lhs = this.is_true();
            let rhs = args[0].is_true();
            lhs || rhs
        })
        .set_op_impl(Operator::Add, |_, this, args| this + args[0].clone())
        .set_op_impl(Operator::Subtract, |_, this, args| this - args[0].clone())
        .set_op_impl(Operator::Times, |_, this, args| this * args[0].clone())
        .set_op_impl(Operator::Divide, |_, this, args| this / args[0].clone())
        .set_op_impl(Operator::Equal, |_, this, args| this == args[0])
        .set_op_impl(Operator::GreaterThan, |_, this, args| this > args[0])
        .set_op_impl(Operator::GreaterThanOrEqual, |_, this, args| {
            this >= args[0]
        })
        .set_op_impl(Operator::LessThan, |_, this, args| this < args[0])
        .set_op_impl(Operator::LessThanOrEqual, |_, this, args| this <= args[0])
        .set_op_impl(Operator::NotEqual, |_, this, args| this != args[0])
        .set_op_impl(Operator::LessThanOrEqual, |_, this, args| this <= args[0])
    }

    pub fn add_field(mut self, name: &str, default: Expression) -> Self {
        self.fields.insert(name.to_string(), default);
        self
    }

    pub fn add_function<T: Into<Dynamic>>(
        mut self,
        name: &str,
        f: impl Fn(&mut Interpreter, Dynamic, Vec<Dynamic>) -> T + 'static,
    ) -> Self {
        self.functions.insert(
            name.to_string(),
            Method::new(name, move |a, b, c| f(a, b, c).into()),
        );
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

    pub fn set_op_impl<T: Into<Dynamic>>(
        mut self,
        op: Operator,
        f: impl Fn(&mut Interpreter, Dynamic, Vec<Dynamic>) -> T + 'static,
    ) -> Self {
        let op_method_name = op.method_name();
        self.op_impls.insert(
            op,
            Method::new(&op_method_name, move |a, b, c| f(a, b, c).into()),
        );
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
