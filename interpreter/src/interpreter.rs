use super::{
    ast::Ast,
    ast::ClassMember,
    class::Class,
    dynamic::{object_builder::ObjectBuilder, Dynamic},
    expression::Expression,
    function::Function,
    module::Module,
    operator::Operator,
    parser::Parser,
    scope::Scope,
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
    pub mappings: HashMap<String, Dynamic>,
}

#[derive(Debug)]
pub struct Interpreter {
    pub state: InterpreterState,
    /// Contains the current file path to the file being interpreted
    pub file_path: PathBuf,
    /// This is true if a break statement was encountered until it is consumed
    pub broken: bool,
    /// This is true if a continue statement was encountered until it is consumed
    pub continued: bool,
    pub default_classes: HashMap<String, Class>,
    pub classes: HashMap<String, Class>,
    pub default_variables: HashMap<String, Dynamic>,
    pub exported_variables: Vec<String>,
    pub exported_classes: Vec<String>,
    pub module_cache: HashMap<PathBuf, Module>,
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
            broken: false,
            continued: false,
            default_classes: create_default_classes(),
            default_variables: create_default_variables(),
            classes: HashMap::new(),
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
        self.classes
            .get(name)
            .or_else(|| self.default_classes.get(name))
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

    fn with_clean_state<T>(
        &mut self,
        scope: Scope,
        new_file_path: Option<PathBuf>,
        f: impl Fn(&mut Interpreter) -> T,
    ) -> T {
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
        self.classes = self.classes.clone().into_iter().chain(classes).collect();
        self.return_expr = return_expr;
        self.file_path = file_path;

        result
    }

    fn assign_variable(&mut self, name: String, value: Dynamic) {
        let mut path = name.split(".").peekable();
        let root_path = path.next().unwrap();
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
                scope.set(name, value);
            }
        } else {
            panic!("Variable {} doesn't exist!", name);
        }
    }

    fn eval(&mut self, expr: &Expression) -> Dynamic {
        match expr {
            Expression::PreOp(op, rhs) => {
                let value = self.eval(rhs);
                match op.as_str() {
                    "-" => match value {
                        Dynamic::Number(x) => (-x).into(),
                        _ => Dynamic::Null
                    },
                    "+" => match value {
                        Dynamic::Number(x) => (x).into(),
                        _ => Dynamic::Null
                    },
                    "!" => (!value.is_true()).into(),
                    _ => Dynamic::Null
                }
            },
            Expression::PostOp(lhs, op, arg) => {
                let op = Operator::from_str(op).unwrap();
                let value = self.eval(lhs);

                let arg = arg.as_ref().map(|arg| self.eval(arg.as_ref()));

                let class = self.find_class(&value.type_name()).unwrap().clone();

                if let Some(cb) = class.get_op_impl(&op) {
                    let res = cb.invoke(
                        self,
                        value,
                        match op {
                            Operator::Call => arg.and_then(|x| x.as_array()).unwrap(),
                            _ => vec![arg.unwrap_or_default()],
                        },
                    );

                    match op {
                        Operator::Increment | Operator::Decrement => {
                            let ident = lhs.to_string();
                            self.assign_variable(ident, res.clone());
                        }
                        _ => {}
                    };

                    res
                } else {
                    Dynamic::Null
                }
            }
            Expression::BinaryOp(lhs, op, rhs) => {
                let op = Operator::from_str(op).unwrap();
                let (class_name, is_static) = match lhs.as_ref() {
                    Expression::ClassIdentifier(x) => (Some(x), true),
                    _ => (None, false),
                };
                let (lhs, args) = match op {
                    Operator::Dot => match rhs.as_ref() {
                        Expression::ClassIdentifier(x) | Expression::Identifier(x) => {
                            (self.eval(lhs.as_ref()), vec![Dynamic::String(x.clone())])
                        }
                        _ => unreachable!(rhs),
                    },
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
                    let field_name = args[0].clone().as_str().unwrap();

                    if let Some(f) = class.static_functions.get(&field_name).cloned() {
                        f.into()
                    } else {
                        panic!(
                            "The class {} doesn't have a static function called {}",
                            class_name.unwrap(),
                            field_name
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
            Expression::ArrowFunction(arg_names, body) => Dynamic::Function {
                name: "<anonymous function>".into(),
                arg_names: arg_names.clone(),
                body: body.clone(),
                scope: (&self.scopes).into(),
            },
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
        scope: Option<Scope>,
        arg_names: &Vec<String>,
        args: &Vec<Dynamic>,
        body: &Vec<Ast>,
    ) -> Dynamic {
        let mut f_scope = scope.unwrap_or_default();
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
            Ast::Documentation(_) => {}
            Ast::VariableAssignment(name, value) => {
                let value = self.eval(value);
                self.assign_variable(name.clone(), value)
            }
            Ast::FunctionCall(name, arg_values) => match self.find(name).clone() {
                Dynamic::Function {
                    arg_names,
                    body,
                    scope,
                    ..
                } => {
                    self.call_fn(
                        None,
                        Some(scope),
                        &arg_names,
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
                let flat_scope = (&self.scopes).into();
                let scope = self.get_scope_mut();
                scope.set(
                    name.clone(),
                    Dynamic::Function {
                        name: name.clone(),
                        arg_names: args.clone(),
                        body: body.clone(),
                        scope: flat_scope,
                    },
                )
            }
            Ast::IfStatement(branches) => {
                for (cond, block) in branches {
                    if self.eval(cond).is_true() {
                        self.scopes.push(Scope::default());
                        self.execute_stmts(block);
                        self.scopes.pop();
                        break;
                    }
                }
            }
            Ast::WhileStatement(cond, block) => {
                while !self.broken && self.eval(cond).is_true() {
                    self.scopes.push(Scope::default());
                    self.execute_stmts(block);
                    self.scopes.pop();
                    self.continued = false;
                }
                self.broken = false;
            }
            Ast::ClassDefinition(name, members) => {
                let mut class = Class::new(&name);

                for member in members {
                    match member {
                        ClassMember::StaticFunction(name, arg_names, body) => {
                            let body = body.clone();
                            let arg_names = arg_names.clone();
                            class = class.add_static_function(name, move |interp, args| {
                                //TODO: also capture scope
                                interp.call_fn(None, None, &arg_names, &args, &body)
                            });
                        }
                        ClassMember::Function(name, arg_names, body) => {
                            let body = body.clone();
                            let arg_names = arg_names.clone();
                            class = class.add_function(name, move |interp, this, args| {
                                //TODO: also capture scope
                                interp.call_fn(Some(this), None, &arg_names, &args, &body)
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
            Ast::PlusAssignment(name, expr) => {
                let new_value = self.eval(
                    &Expression::BinaryOp(
                        Box::new(Expression::Identifier(name.into())), 
                        "+".into(), 
                        Box::new(expr.clone())
                    )
                );
                self.assign_variable(name.clone(), new_value);
            },
            Ast::MinusAssignment(name, expr) => {
                let new_value = self.eval(
                    &Expression::BinaryOp(
                        Box::new(Expression::Identifier(name.into())), 
                        "-".into(), 
                        Box::new(expr.clone())
                    )
                );
                self.assign_variable(name.clone(), new_value);
            },
            Ast::TimesAssignment(name, expr) => {
                let new_value = self.eval(
                    &Expression::BinaryOp(
                        Box::new(Expression::Identifier(name.into())), 
                        "*".into(), 
                        Box::new(expr.clone())
                    )
                );
                self.assign_variable(name.clone(), new_value);
            },
            Ast::DivideAssignment(name, expr) => {
                let new_value = self.eval(
                    &Expression::BinaryOp(
                        Box::new(Expression::Identifier(name.into())), 
                        "/".into(), 
                        Box::new(expr.clone())
                    )
                );
                self.assign_variable(name.clone(), new_value);
            },
            Ast::BreakStatement => {
                self.broken = true;
            }
            Ast::ContinueStatement => {
                self.continued = true;
            }
            Ast::Expression(expr) => {
                self.eval(expr);
            }
            Ast::StaticFunctionDefinition(_, _, _) => unreachable!(),
            Ast::ImportStatement(path) => {
                let mod_name = self.module_path_to_name(path);
                let file_path = self.module_path_to_file_path(path);
                if let Some(module) = self.module_cache.get(&file_path).cloned() {
                    self.get_scope_mut().set(mod_name, Dynamic::Module(module));
                } else {
                    let module =
                        self.with_clean_state(Scope::default(), Some(file_path.clone()), |i| {
                            let mut parser = Parser::new();
                            let content = std::fs::read_to_string(&file_path).unwrap();

                            parser.set_source(file_path.clone(), &content);

                            let program = parser.parse();
                            let module = i.execute(&program);

                            i.module_cache.insert(file_path.clone(), module.clone());

                            module
                        });

                    self.get_scope_mut().set(mod_name, Dynamic::Module(module));
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
                    Ast::VariableDefinition(name, _) => {
                        self.exported_variables.push(name.clone());
                        self.execute_stmt(ast);
                    }
                    Ast::FunctionDefinition(name, _, _) => {
                        self.exported_variables.push(name.clone());
                        self.execute_stmt(ast);
                    }
                    Ast::ClassDefinition(name, _) => {
                        self.exported_classes.push(name.clone());
                        self.execute_stmt(ast);
                    }
                    _ => todo!(),
                };
            }
        }
    }

    fn execute_stmts(&mut self, stmts: &Vec<Ast>) {
        for stmt in stmts {
            self.execute_stmt(stmt);
            if self.return_expr.is_some() || self.broken || self.continued {
                break;
            }
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
                Dynamic::Function {
                    body,
                    arg_names,
                    scope,
                    ..
                } => {
                    let arg_names = arg_names.clone();
                    let body = body.clone();
                    let value = Function::new(&var_name, Some(scope), move |interp, args| {
                        interp.call_fn(None, None, &arg_names, &args, &body)
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
        Class::new("number")
            .set_op_impl(Operator::Increment, |_, this, _| number!(this) + 1)
            .set_op_impl(Operator::Decrement, |_, this, _| number!(this) - 1),
    );
    classes.push(Class::new("string"));
    classes.push(
        Class::new("array")
            .add_function("push", |_, this, args| {
                if let Dynamic::Array(items_ref) = &this {
                    for arg in args {
                        items_ref.lock().unwrap().push(arg.clone());
                    }
                } else {
                    unreachable!()
                }

                this
            })
            .set_op_impl(Operator::Index, |_interp, this, args| {
                let this_ref = array!(this);
                let this = this_ref.lock().unwrap();
                let other = number!(args[0]);

                this.get(other as usize).cloned().unwrap_or_default()
            }),
    );
    classes.push(Class::new("null"));
    classes.push(Class::new("module"));
    classes.push(
        Class::new("class").set_op_impl(Operator::Constructor, |i, this, args| {
            if let Dynamic::Class(this) = this {
                Dynamic::new_instance(&this.name, Default::default())
            } else {
                unreachable!()
            }
        }),
    );
    classes.push(Class::new("object"));
    classes.push(Class::new("boolean"));
    classes.push(
        Class::new("function").set_op_impl(Operator::Call, |i, this, args| {
            if let Dynamic::Function {
                arg_names,
                scope,
                body,
                ..
            } = this
            {
                i.call_fn(None, Some(scope), &arg_names, &args, &body)
            } else {
                unreachable!();
            }
        }),
    );
    classes.push(
        Class::new("extern function").set_op_impl(Operator::Call, |i, this, args| {
            if let Dynamic::RustFunction { callback, .. } = this {
                callback(i, args).unwrap_or_default()
            } else {
                unreachable!();
            }
        }),
    );

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
        .function("range", |_, args| {
            Some(match args.len() {
                1 => {
                    let count = number!(args[0]);
                    let mut items = Vec::new();
                    for i in 0..count {
                        items.push(Dynamic::Number(i));
                    }
                    Dynamic::new_array(items)
                },
                2 => {
                    let start = number!(args[0]);
                    let count = number!(args[1]);
                    let mut items = Vec::new();
                    for i in start..start+count {
                        items.push(Dynamic::Number(i));
                    }
                    Dynamic::new_array(items)
                },
                _ => todo!()
            })
        })
        .build()
}
