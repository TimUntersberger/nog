use super::{
    ast::Ast, ast::ClassMember, class::Class, dynamic::{Dynamic, object_builder::ObjectBuilder}, expression::Expression,
    function::Function, module::Module, operator::Operator, parser::Parser, scope::Scope,
};
use itertools::Itertools;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

#[derive(Debug)]
pub struct Program {
    pub path: PathBuf,
    pub stmts: Vec<Ast>,
}

#[derive(Debug, Default)]
pub struct InterpreterState {
    pub mappings: HashMap<String, Dynamic>
}

#[derive(Debug)]
pub struct Interpreter {
    pub state: InterpreterState,
    /// Contains the current file path to the file being interpreted
    pub file_path: PathBuf,
    pub default_classes: HashMap<String, Class>,
    pub classes: HashMap<String, Class>,
    pub default_variables: HashMap<String, Dynamic>,
    pub exported_variables: Vec<String>,
    pub exported_classes: Vec<String>,
    pub module_cache: HashMap<PathBuf, Module>,
    pub modules: HashMap<String, Module>,
    /// This may contain an expression if a return statement was parsed. This gets consumed when a
    /// function definition finishes parsing
    pub return_expr: Option<Expression>,
    /// This represents the scope hierachy where the scope at index 0 is the global scope and every
    /// scope after the first one is a subscope of the previous one
    pub scopes: Vec<Scope>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            state: Default::default(),
            file_path: Default::default(),
            default_classes: create_default_classes(),
            default_variables: create_default_variables(),
            classes: HashMap::new(),
            modules: HashMap::new(),
            module_cache: HashMap::new(),
            exported_classes: Vec::new(),
            exported_variables: Vec::new(),
            return_expr: None,
            scopes: vec![Scope::default()],
        }
    }
    fn module_path_to_file_path(&self, module_path: &str) -> PathBuf {
        let mut path = PathBuf::new();

        path.push(&self.file_path.parent().unwrap());

        for part in module_path.split(".") {
            path.push(part);
        }

        path.set_extension("nog");

        path
    }

    fn module_path_to_name(&self, module_path: &str) -> String {
        module_path.split(".").last().unwrap().to_string()
    }

    fn find_class(&self, name: &str) -> Option<&Class> {
        self.classes.get(name).or_else(|| self.default_classes.get(name))
    }

    pub fn add_class(&mut self, class: Class) {
        self.classes.insert(class.name.clone(), class);
    }
    pub fn get_scope(&self) -> &Scope {
        self.scopes.iter().last().unwrap()
    }
    pub fn get_scope_mut(&mut self) -> &mut Scope {
        self.scopes.iter_mut().last().unwrap()
    }

    fn with_clean_state<T>(&mut self, scope: Scope, new_file_path: Option<PathBuf>, f: impl Fn(&mut Interpreter) -> T) -> T {
        let file_path = self.file_path.clone();
        let scopes = self.scopes.clone();
        let classes = self.classes.clone();
        let return_expr = self.return_expr.clone();

        self.file_path = new_file_path.unwrap_or(file_path.clone());
        self.return_expr = None;
        self.classes = HashMap::new();
        self.scopes = vec![scope];

        let result = f(self);

        self.scopes = scopes;
        self.classes = classes;
        self.return_expr = return_expr;
        self.file_path = file_path;

        result
    }

    fn eval(&mut self, expr: &Expression) -> Dynamic {
        match expr {
            Expression::BinaryOp(lhs, op, rhs) => {
                let op = Operator::from_str(op).unwrap();
                let (class_name, is_static) = match lhs.as_ref() {
                    Expression::ClassIdentifier(x) => (Some(x), true),
                    _ => (None, false),
                };
                let (lhs, args) = match op {
                    Operator::Dot => match rhs.as_ref() {
                        Expression::Identifier(x) => {
                            (self.eval(lhs.as_ref()), vec![Dynamic::String(x.clone())])
                        }
                        Expression::FunctionCall(name, args) => {
                            if is_static {
                                (
                                    Dynamic::Null,
                                    vec![Dynamic::new_array(vec![
                                        name.clone().into(),
                                        args.clone().into(),
                                    ])],
                                )
                            } else {
                                (
                                    self.eval(lhs.as_ref()),
                                    vec![Dynamic::new_array(vec![
                                        name.clone().into(),
                                        args.clone().into(),
                                    ])],
                                )
                            }
                        }
                        _ => unreachable!(rhs),
                    },
                    Operator::Namespace => {
                        let module_name = lhs.to_string();
                        let module = self.modules.get(&module_name).cloned();

                        if let Some(module) = module {
                            match rhs.as_ref() {
                                Expression::FunctionCall(fn_name, args) => {
                                    if let Some(function) = module.functions.get(fn_name) {
                                        let arg_values = args
                                            .iter()
                                            .map(|expr| self.eval(expr))
                                            .collect::<Vec<Dynamic>>();
                                        return self.with_clean_state(module.scope, None, |i| {
                                            function.invoke(i, arg_values.clone())
                                        });
                                    } else {
                                        panic!(
                                            "Function with name {} doesn't exist in {}",
                                            fn_name, module_name
                                        );
                                    }
                                },
                                Expression::Identifier(name) => {
                                    if let Some(variable) = module.variables.get(name) {
                                        return variable.clone();
                                    } else {
                                        panic!(
                                            "Variable with name {} doesn't exist in {}",
                                            name, module_name
                                        );
                                    }
                                }
                                _ => {}
                            };
                        } else {
                            panic!("Module with name {} doesn't exist", module_name);
                        }

                        unreachable!()
                    }
                    Operator::Assign => {
                        let path_str = lhs.to_string();
                        let path_tokens = path_str.split(".").collect::<Vec<&str>>();
                        let path_tokens_len = path_tokens.len();
                        let path = path_tokens[0..path_tokens_len - 1].join(".");
                        let field = path_tokens[path_tokens_len - 1];
                        (
                            self.eval(&Expression::Identifier(path.to_string())),
                            vec![Dynamic::String(field.into()), self.eval(rhs.as_ref())],
                        )
                    }
                    _ => (self.eval(lhs.as_ref()), vec![self.eval(rhs.as_ref())]),
                };

                if is_static {
                    let class = self.find_class(class_name.unwrap()).unwrap();
                    let fn_info = args[0].clone().as_array().unwrap();
                    let fn_name = fn_info[0].clone().as_str().unwrap();

                    if let Some(f) = class.static_functions.get(&fn_name).cloned() {
                        f.invoke(self, args)
                    } else {
                        panic!(
                            "The class {} doesn't have a static function called {}",
                            class_name.unwrap(),
                            fn_name
                        );
                    }
                } else {
                    let class = self.find_class(&lhs.type_name()).unwrap();

                    if let Some(f) = class.get_op_impl(&op).cloned() {
                        f.invoke(self, lhs, args)
                    } else {
                        panic!(
                            "The class {} doesn't implement the operator {}",
                            class.name,
                            op.to_string()
                        );
                    }
                }
            }
            Expression::NumberLiteral(x) => Dynamic::Number(x.clone()),
            Expression::BooleanLiteral(x) => Dynamic::Boolean(x.clone()),
            Expression::StringLiteral(x) => Dynamic::String(x.clone()),
            Expression::Null => Dynamic::Null,
            Expression::Identifier(key) => self.find(&key).clone(),
            Expression::ClassIdentifier(name) => self
                .classes
                .get(name)
                .map(|c| c.clone().into())
                .unwrap_or_default(),
            Expression::FunctionCall(name, arg_values) => {
                let val = self.find(&name).clone();
                match val {
                    Dynamic::Function { args, body, .. } => self.call_fn(
                        None,
                        &args,
                        &arg_values.iter().map(|a| a.into()).collect(),
                        &body,
                    ),
                    Dynamic::RustFunction { name, callback } => {
                        let args = arg_values.iter().map(|a| self.eval(a)).collect();
                        callback(self, args).unwrap_or_default()
                    }
                    actual => panic!("Expected {} to be a function, but it was {}", name, actual),
                }
            }
            Expression::ClassInstantiation(name, values) => {
                if let Some(class_fields) = self.find_class(name).map(|c| c.fields.clone()) {
                    let mut fields = HashMap::new();
                    for (name, default) in class_fields {
                        fields.insert(
                            name.clone(),
                            self.eval(&values.get(&name).unwrap_or(&default)),
                        );
                    }
                    Dynamic::new_instance(name, fields)
                } else {
                    panic!("Class {} doesn't exist", name);
                }
            }
            Expression::ArrayLiteral(items) => {
                Dynamic::new_array(items.iter().map(|expr| self.eval(expr)).collect())
            }
            Expression::ObjectLiteral(fields) => Dynamic::new_object(
                fields
                    .into_iter()
                    .map(|(k, v)| (k.clone(), self.eval(v)))
                    .collect(),
            ),
        }
    }
    fn consume_return_value(&mut self) -> Dynamic {
        let result = self
            .return_expr
            .clone()
            .map(|expr| self.eval(&expr))
            .unwrap_or_default();
        self.return_expr = None;
        result
    }
    pub fn call_fn(
        &mut self,
        this: Option<Dynamic>,
        arg_names: &Vec<String>,
        args: &Vec<Dynamic>,
        body: &Vec<Ast>,
    ) -> Dynamic {
        let mut f_scope = Scope::default();
        for (arg_name, arg) in arg_names.iter().zip(args.iter()) {
            f_scope.set(
                arg_name.clone(),
                match arg {
                    Dynamic::Lazy(expr) => self.eval(expr),
                    x => x.clone(),
                },
            );
        }
        if let Some(this) = this {
            f_scope.set("this".to_string(), this);
        }
        self.scopes.push(f_scope);
        self.execute_stmts(&body);
        let result = self.consume_return_value();
        self.scopes.pop();
        result
    }

    fn find(&mut self, key: &str) -> Dynamic {
        let mut path = key.split(".").peekable();
        let root_path = path.next().unwrap();
        if let Some(scope) = self
            .scopes
            .iter_mut()
            .rev()
            .find(|s| s.is_defined(root_path))
        {
            let mut field_value = scope.get(root_path).clone();
            if let Some(ident) = path.next() {
                let mut field_ident = ident;
                loop {
                    field_value = field_value.get_field(field_ident);
                    field_ident = match path.next() {
                        Some(x) => x,
                        None => break,
                    };
                }
            }
            field_value
        } else {
            let field_value = self.default_variables.get(root_path);
            if let Some(mut field_value) = field_value.cloned() {
                if let Some(ident) = path.next() {
                    let mut field_ident = ident;
                    loop {
                        field_value = field_value.get_field(field_ident);
                        field_ident = match path.next() {
                            Some(x) => x,
                            None => break,
                        };
                    }
                }
            }
            field_value.unwrap_or_default().clone()
        }
    }
    fn execute_stmt(&mut self, stmt: &Ast) {
        match stmt {
            Ast::VariableDefinition(name, value) => {
                let value = self.eval(value);
                self.get_scope_mut().set(name.clone(), value)
            }
            Ast::VariableAssignment(name, value) => {
                let mut path = name.split(".").peekable();
                let root_path = path.next().unwrap();
                let value = self.eval(value);

                if let Some(scope) = self
                    .scopes
                    .iter_mut()
                    .rev()
                    .find(|s| s.is_defined(root_path))
                {
                    if let Some(ident) = path.next() {
                        let mut field_value = scope.get(root_path).clone();
                        let mut field_ident = ident;
                        loop {
                            if path.peek().is_none() {
                                field_value.set_field(field_ident, value);
                                break;
                            }
                            field_value = field_value.get_field(field_ident);
                            field_ident = match path.next() {
                                Some(x) => x,
                                None => break,
                            };
                        }
                    } else {
                        scope.set(name.clone(), value);
                    }
                } else {
                    panic!("Variable {} doesn't exist!", name);
                }
            }
            Ast::FunctionCall(name, arg_values) => match self.find(name).clone() {
                Dynamic::Function { args, body, .. } => {
                    self.call_fn(
                        None,
                        &args,
                        &arg_values.iter().map(|a| a.into()).collect(),
                        &body,
                    );
                }
                Dynamic::RustFunction { callback, .. } => {
                    let args = arg_values.iter().map(|a| self.eval(a)).collect();
                    callback(self, args).unwrap_or_default();
                }
                actual => panic!("Expected {} to be a function, but it is a {}", name, actual),
            },
            Ast::FunctionDefinition(name, args, body) => {
                let scope = self.get_scope_mut();
                scope.set(
                    name.clone(),
                    Dynamic::Function {
                        name: name.clone(),
                        args: args.clone(),
                        body: body.clone(),
                    },
                )
            }
            Ast::IfStatement(cond, block) => {
                if self.eval(cond).is_true() {
                    self.scopes.push(Scope::default());
                    self.execute_stmts(block);
                    self.scopes.pop();
                }
            }
            Ast::ClassDefinition(name, members) => {
                let mut class = Class::new(&name);

                for member in members {
                    match member {
                        ClassMember::StaticFunction(name, arg_names, body) => {
                            let body = body.clone();
                            let arg_names = arg_names.clone();
                            class = class.add_static_function(name, move |interp, args| {
                                interp.call_fn(None, &arg_names, &args, &body)
                            });
                        }
                        ClassMember::Function(name, arg_names, body) => {
                            let body = body.clone();
                            let arg_names = arg_names.clone();
                            class = class.add_function(name, move |interp, this, args| {
                                interp.call_fn(Some(this), &arg_names, &args, &body)
                            });
                        }
                        ClassMember::Field(name, default) => {
                            class = class.add_field(name, default.clone());
                        }
                        ClassMember::Operator(op, arg_names, body) => {
                            let body = body.clone();
                            let args = arg_names.clone();
                            class =
                                class.set_op_impl(op.clone(), move |interp, this, arg_values| {
                                    let mut f_scope = Scope::default();
                                    for (arg_name, expr) in args.iter().zip(arg_values.into_iter())
                                    {
                                        f_scope.set(arg_name.clone(), expr.clone());
                                    }
                                    f_scope.set("this".into(), this);
                                    interp.scopes.push(f_scope);
                                    interp.execute_stmts(&body);
                                    let result = interp.consume_return_value();
                                    interp.scopes.pop();
                                    result
                                });
                        }
                    }
                }

                self.add_class(class)
            }
            Ast::OperatorImplementation(_, _, _) => unreachable!(),
            Ast::ReturnStatement(expr) => {
                self.return_expr = Some(expr.clone());
            }
            Ast::Expression(expr) => {
                self.eval(expr);
            }
            Ast::StaticFunctionDefinition(_, _, _) => unreachable!(),
            Ast::ImportStatement(path) => {
                let mod_name = self.module_path_to_name(path);
                let file_path = self.module_path_to_file_path(path);
                if let Some(module) = self.module_cache.get(&file_path) {
                    self.modules.insert(mod_name.clone(), module.clone());
                } else {
                    self.with_clean_state(Scope::default(), Some(file_path.clone()), |i| {
                        let mut parser = Parser::new();
                        let content = std::fs::read_to_string(&file_path).unwrap();

                        parser.set_source(file_path.clone(), &content);

                        let program = parser.parse();
                        let module = i.execute(&program);

                        i.module_cache.insert(file_path.clone(), module.clone());
                        i.modules.insert(mod_name.clone(), module);
                    });
                }
            }
            Ast::ExportStatement(ast) => {
                match ast.as_ref() {
                    Ast::Expression(expr) => match expr {
                        Expression::Identifier(x) => {
                            self.exported_variables.push(x.clone());
                        }
                        Expression::ClassIdentifier(x) => {
                            self.exported_classes.push(x.clone());
                        }
                        _ => unreachable!(),
                    },
                    _ => todo!(),
                };
            }
        }

    }

    fn execute_stmts(&mut self, stmts: &Vec<Ast>) {
        for stmt in stmts {
            self.execute_stmt(stmt);
        }
    }

    pub fn execute(&mut self, prog: &Program) -> Module {
        self.file_path = prog.path.clone();
        self.execute_stmts(&prog.stmts);

        let mut variables = HashMap::new();
        let mut classes = HashMap::new();
        let mut functions = HashMap::new();

        for var_name in self.exported_variables.clone() {
            let value = self.find(&var_name);
            match value {
                Dynamic::Function { body, args, .. } => {
                    let arg_names = args.clone();
                    let body = body.clone();
                    let value = Function::new(&var_name, move |interp, args| {
                        interp.call_fn(None, &arg_names, &args, &body)
                    });
                    functions.insert(var_name, value);
                }
                _ => {
                    variables.insert(var_name, value);
                }
            };
        }

        for class_name in self.exported_classes.clone() {
            let value = self.find_class(&class_name).unwrap();
            classes.insert(class_name, value.clone());
        }

        Module {
            name: "".into(),
            variables,
            scope: self.scopes.first().unwrap().clone(),
            functions,
            classes,
        }
    }
}

fn create_default_classes() -> HashMap<String, Class> {
    let mut classes = Vec::new();

    classes.push(
        Class::new("number").set_op_impl(Operator::Add, |_, this, args| {
            let rhs = &args[0];

            if let Dynamic::Number(this) = this {
                match rhs {
                    Dynamic::Number(x) => Dynamic::Number(this + x),
                    _ => todo!(),
                }
            } else {
                unreachable!()
            }
        }),
    );
    classes.push(
        Class::new("string").set_op_impl(Operator::Add, |_, this, args| {
            let rhs = &args[0];

            if let Dynamic::String(this) = this {
                match rhs {
                    Dynamic::String(x) => Dynamic::String(format!("{}{}", this, x)),
                    Dynamic::Number(x) => Dynamic::String(format!("{}{}", this, x)),
                    Dynamic::Boolean(x) => Dynamic::String(format!("{}{}", this, x)),
                    _ => todo!(),
                }
            } else {
                unreachable!()
            }
        }),
    );
    classes.push(Class::new("array").add_function("push", |_, this, args| {
        if let Dynamic::Array(items_ref) = &this {
            for arg in args {
                items_ref.lock().unwrap().push(arg.clone());
            }
        } else {
            unreachable!()
        }

        this
    }));
    classes.push(Class::new("object"));
    classes.push(Class::new("boolean"));
    classes.push(Class::new("function"));
    classes.push(Class::new("extern function"));

    classes.into_iter().map(|c| (c.name.clone(), c)).collect()
}

pub fn create_default_variables() -> HashMap<String, Dynamic> {
    ObjectBuilder::new()
        .function("print", |_, args| {
            println!("{}", args.iter().join(" "));
            None
        })
        .function("typeof", |_, args| {
            args.get(0).map(|a| a.type_name().into())
        })
        .object("nog", ObjectBuilder::new()
                .string("version", "<VERSION>")
                .function("map", |i, args| {
                    match &args[0] {
                        Dynamic::Lazy(expr) => {
                            let key = i.eval(&expr).as_str().unwrap();
                            match &args[1] {
                                Dynamic::Lazy(expr) => {
                                    let f = i.eval(&expr);
                                    i.state.mappings.insert(key, f);
                                },
                                _ => todo!()
                            }
                        },
                        _ => todo!()
                    }

                    None
                })
                .function("mode", |i, args| {
                    // Usage 
                    // 
                    // nog.mode("resize", () => {
                    //     nog.map("Alt+H", () => nog.api.window.resize_left(2))
                    // });
                    todo!();
                    None
                })
                .object("api", ObjectBuilder::new()
                        .object("mode", ObjectBuilder::new()
                                .function("get_current", |_, args| todo!())
                                .function("leave", |_, args| todo!())
                                .function("enter", |_, args| todo!())
                                .build())
                        .object("config", ObjectBuilder::new()
                                .build())
                        .object("workspace", ObjectBuilder::new()
                                .build())
                        .object("window", ObjectBuilder::new()
                                .build())
                        .build())
                .build())
        .build()
}
