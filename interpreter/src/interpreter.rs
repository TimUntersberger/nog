use super::{
    ast::ClassMember,
    ast::{AstKind, AstNode},
    class::Class,
    dynamic::{object_builder::ObjectBuilder, Dynamic, Number},
    expression::{Expression, ExpressionKind},
    formatter::Formatter,
    function::Function,
    lexer::Lexer,
    module::Module,
    operator::Operator,
    parser::Parser,
    runtime_error::*,
    scope::Scope,
    token::{Token, TokenKind},
};
use itertools::Itertools;
use std::{collections::HashMap, iter, ops::Range, path::PathBuf, sync::Arc, time::Instant};

#[derive(Debug)]
pub struct Program<'a> {
    pub path: PathBuf,
    pub source: &'a str,
    pub stmts: Vec<AstNode>,
}

impl<'a> Default for Program<'a> {
    fn default() -> Self {
        Self {
            path: Default::default(),
            source: Default::default(),
            stmts: Default::default(),
        }
    }
}

impl<'a> Program<'a> {
    pub fn print(&self) {
        println!(
            "file  {}\n",
            iter::once(self.path.to_str().unwrap().to_string())
                .chain(
                    Formatter::new(self)
                        .format()
                        .split("\n")
                        .enumerate()
                        .map(|(i, line)| format!("{:03} | {}", i + 1, line))
                )
                .join("\n")
        );
    }

    pub fn range_to_location(&self, range: Range<usize>) -> Option<(usize, usize)> {
        let mut line = 1;
        let mut col_start = 0;

        for (idx, c) in self.source.char_indices() {
            if idx >= range.start {
                return Some((line, idx - col_start + 1));
            }
            if c == '\n' {
                line += 1;
                col_start = idx;
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct Interpreter {
    /// Contains a list of directories, where the interpreter searches for source files when
    /// importing
    pub source_locations: Vec<PathBuf>,
    /// Contains the current file path to the file being interpreted
    pub file_path: PathBuf,
    /// Contains the stmts being executed
    pub stmts: Vec<AstNode>,
    /// Whether to print debug information
    pub debug: bool,
    pub source: String,
    /// This is true if a break statement was encountered until it is consumed
    pub broken: bool,
    /// This is true if a continue statement was encountered until it is consumed
    pub continued: bool,
    pub default_classes: HashMap<String, Class>,
    pub modules: HashMap<String, Module>,
    pub classes: HashMap<String, Class>,
    pub default_variables: HashMap<String, Dynamic>,
    pub exported_variables: Vec<String>,
    pub exported_classes: Vec<String>,
    pub module_cache: HashMap<PathBuf, Module>,
    /// This may contain a dynamic if a return statement was parsed. This gets consumed when a
    /// function definition finishes parsing
    pub return_value: Option<Dynamic>,
    /// This represents the scope hierachy where the scope at index 0 is the global scope and every
    /// scope after the first one is a subscope of the previous one
    pub scopes: Vec<Scope>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            source_locations: Vec::new(),
            file_path: Default::default(),
            source: Default::default(),
            stmts: Vec::new(),
            broken: false,
            debug: false,
            continued: false,
            default_classes: create_default_classes(),
            default_variables: create_default_variables(),
            modules: create_default_modules(),
            classes: HashMap::new(),
            module_cache: HashMap::new(),
            exported_classes: Vec::new(),
            exported_variables: Vec::new(),
            return_value: None,
            scopes: vec![Scope::default()],
        }
    }

    fn module_path_to_file_path(&self, module_path: &str) -> Option<PathBuf> {
        for dir_path in &self.source_locations {
            let mut path = PathBuf::new();

            path.push(dir_path);

            for part in module_path.split(".") {
                path.push(part);
            }

            path.set_extension("ns");

            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    fn module_path_to_name(&self, module_path: &str) -> String {
        module_path.split(".").last().unwrap().to_string()
    }

    /// Builds a Program using the current state
    pub fn program(&self) -> Program {
        Program {
            stmts: self.stmts.clone(),
            source: &self.source,
            path: self.file_path.clone(),
        }
    }

    pub fn find_class(&self, name: &str) -> Option<&Class> {
        self.classes
            .get(name)
            .or_else(|| self.default_classes.get(name))
    }

    pub fn add_class(&mut self, class: Class) {
        self.classes.insert(class.name.clone(), class);
    }
    pub fn add_module(&mut self, module: Module) {
        self.modules.insert(module.name.clone(), module);
    }
    pub fn get_scope(&self) -> &Scope {
        self.scopes.iter().last().unwrap()
    }
    pub fn get_scope_mut(&mut self) -> &mut Scope {
        self.scopes.iter_mut().last().unwrap()
    }

    pub fn instantiate_class(
        &mut self,
        name: &str,
        values: &HashMap<String, Dynamic>,
    ) -> RuntimeResult {
        if let Some(class_fields) = self.find_class(name).map(|c| c.fields.clone()) {
            let mut fields = HashMap::new();
            for (name, default) in class_fields {
                fields.insert(
                    name.clone(),
                    values
                        .get(&name)
                        .map(|x| Ok(x.clone()))
                        .unwrap_or_else(|| self.eval(&default))?,
                );
            }
            Ok(Dynamic::new_instance(name, fields))
        } else {
            Err(RuntimeError::ClassNotFound {
                name: name.to_string(),
            })
        }
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
        let return_value = self.return_value.clone();

        self.file_path = new_file_path.unwrap_or(file_path.clone());
        self.return_value = None;
        self.classes = HashMap::new();
        self.scopes = vec![scope];

        let result = f(self);

        self.scopes = scopes;
        self.classes = self.classes.clone().into_iter().chain(classes).collect();
        self.return_value = return_value;
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

    fn text(&self, token: &Token) -> &str {
        &self.source[token.1.clone()]
    }

    fn eval(&mut self, expr: &Expression) -> RuntimeResult {
        match &expr.kind {
            ExpressionKind::PreOp(op, rhs) => {
                let value = self.eval(rhs.as_ref())?;
                Ok(match op {
                    Operator::Subtract => match value {
                        Dynamic::Number(x) => (-x).into(),
                        _ => Dynamic::Null,
                    },
                    Operator::Add => match value {
                        Dynamic::Number(x) => (x).into(),
                        _ => Dynamic::Null,
                    },
                    Operator::Not => (!value.is_true()).into(),
                    _ => Dynamic::Null,
                })
            }
            ExpressionKind::PostOp(lhs, op, arg) => {
                let value = self.eval(lhs.as_ref())?;

                let arg = arg.as_ref().map(|arg| self.eval(arg.as_ref()));

                let class = self.find_class(&value.type_name()).unwrap().clone();

                if let Some(cb) = class.get_op_impl(&op) {
                    let res = cb.invoke(
                        self,
                        value,
                        match op {
                            Operator::Call => arg.unwrap()?.as_array().unwrap(),
                            _ => vec![arg.unwrap_or(Ok(Default::default()))?],
                        },
                    )?;

                    match op {
                        Operator::Increment | Operator::Decrement => {
                            let ident = lhs.to_string();
                            self.assign_variable(ident, res.clone());
                        }
                        _ => {}
                    };

                    Ok(res)
                } else {
                    Err(RuntimeError::OperatorNotImplemented {
                        expr: expr.clone(),
                        class: class.name,
                        operator: op.clone(),
                    })
                }
            }
            ExpressionKind::BinaryOp(lhs, op, rhs) => {
                let (class_name, is_static) = match &lhs.kind {
                    ExpressionKind::ClassIdentifier(x) => (Some(x), true),
                    _ => (None, false),
                };
                let (lhs, args) = match op {
                    Operator::Dot => match &rhs.kind {
                        ExpressionKind::ClassIdentifier(x) | ExpressionKind::Identifier(x) => {
                            (self.eval(lhs.as_ref())?, vec![Dynamic::String(x.into())])
                        }
                        _ => unreachable!(rhs),
                    },
                    Operator::Assign => {
                        // TODO: SUPPORT COMPLEX ASSIGN LHS
                        // let (lhs, rhs) = match lhs.as_ref() {
                        //     Expression::PostOp(lhs, op, value) => {},
                        //     Expression::BinaryOp(lhs, op, rhs) => {},
                        //     rest => todo!("{:?}", rest)
                        // };
                        let path_str = lhs.to_string();
                        let path_tokens = path_str.split(".").collect::<Vec<&str>>();
                        let path_tokens_len = path_tokens.len();
                        let path = path_tokens[0..path_tokens_len - 1].join(".");
                        let field = path_tokens[path_tokens_len - 1];
                        (
                            self.eval(&Expression::new(
                                ExpressionKind::Identifier(path.to_string()),
                                0..0,
                            ))?,
                            vec![Dynamic::String(field.into()), self.eval(rhs.as_ref())?],
                        )
                    }
                    _ => (self.eval(lhs.as_ref())?, vec![self.eval(rhs.as_ref())?]),
                };

                if is_static {
                    let class = self.find_class(class_name.as_ref().unwrap()).unwrap();
                    let field_name = args[0].clone().as_str().unwrap();

                    if let Some(f) = class.static_functions.get(&field_name).cloned() {
                        Ok(f.into())
                    } else {
                        return Err(RuntimeError::StaticFunctionNotFound {
                            class: class_name.unwrap().to_string(),
                            function_name: field_name,
                        });
                    }
                } else {
                    let class = self.find_class(&lhs.type_name()).unwrap();

                    if class.name == "Null" {
                        return Err(RuntimeError::OperatorNotImplemented {
                            expr: expr.clone(),
                            class: class.name.clone(),
                            operator: op.clone(),
                        });
                    }

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
            ExpressionKind::NumberLiteral(x) => Ok(Dynamic::Number(x.parse().unwrap())),
            ExpressionKind::HexLiteral(x) => {
                Ok(Dynamic::Number(i32::from_str_radix(&x[2..], 16).unwrap()))
            }
            ExpressionKind::BooleanLiteral(x) => Ok(Dynamic::Boolean(x == "true")),
            ExpressionKind::StringLiteral(x) => Ok(Dynamic::String(x.into())),
            ExpressionKind::Null => Ok(Dynamic::Null),
            ExpressionKind::Identifier(key) => Ok(self.find(&key).clone()),
            ExpressionKind::ClassIdentifier(name) => Ok(self
                .find_class(&name)
                .map(|c| c.clone().into())
                .unwrap_or_default()),
            ExpressionKind::ClassInstantiation(name, values) => unreachable!(),
            ExpressionKind::ArrayLiteral(items) => items
                .iter()
                .map(|expr| self.eval(expr))
                .collect::<RuntimeResult<Vec<Dynamic>>>()
                .map(|x| Dynamic::new_array(x)),
            ExpressionKind::ObjectLiteral(fields) => {
                let mut evaluated_fields = HashMap::new();

                for (k, v) in fields {
                    evaluated_fields.insert(k.clone(), self.eval(&v)?);
                }

                Ok(Dynamic::new_object(evaluated_fields))
            }
            ExpressionKind::ArrowFunction(arg_names, body) => Ok(Dynamic::Function {
                name: "<anonymous function>".into(),
                arg_names: arg_names.into_iter().map(|t| t.into()).collect(),
                body: body.clone(),
                scope: (&self.scopes).into(),
            }),
        }
    }
    fn consume_return_value(&mut self) -> Dynamic {
        let result = self.return_value.clone();
        self.return_value = None;
        result.unwrap_or_default()
    }
    pub fn call_fn(
        &mut self,
        this: Option<Dynamic>,
        scope: Option<Scope>,
        arg_names: &Vec<String>,
        args: &Vec<Dynamic>,
        body: &Vec<AstNode>,
    ) -> RuntimeResult {
        let mut f_scope = scope.unwrap_or_default();
        for (arg_name, arg) in arg_names.iter().zip(args.iter()) {
            f_scope.set(
                arg_name.clone(),
                match arg {
                    Dynamic::Lazy(expr) => self.eval(expr)?,
                    x => x.clone(),
                },
            );
        }
        if let Some(this) = this {
            f_scope.set("this".to_string(), this);
        }
        self.scopes.push(f_scope);
        self.execute_stmts(&body)?;
        let result = self.consume_return_value();
        self.scopes.pop();
        Ok(result)
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
    fn import(&mut self, path: &str) -> RuntimeResult<(String, Dynamic)> {
        let mut mod_parts = path.split(".");

        let root_name = mod_parts.next().unwrap();

        let root_mod: Dynamic = match self.modules.get(root_name).cloned() {
            Some(module) => module.into(),
            None => {
                let root_path =
                    self.module_path_to_file_path(path)
                        .ok_or(RuntimeError::ModuleNotFound {
                            name: root_name.to_string(),
                        })?;

                let is_debug = self.debug;
                match self.module_cache.get(&root_path).cloned() {
                    Some(module) => module.into(),
                    None => self
                        .with_clean_state(Scope::default(), Some(root_path.clone()), |i| {
                            let mut parser = Parser::new();
                            let content = std::fs::read_to_string(&root_path).unwrap();

                            parser.set_source(root_path.clone(), &content, 0);

                            let program = parser.parse()?;

                            if is_debug {
                                program.print();
                            }

                            match i.execute(&program) {
                                Ok(module) => {
                                    i.module_cache.insert(root_path.clone(), module.clone());

                                    Ok(module)
                                }
                                x => x,
                            }
                        })?
                        .into(),
                }
            }
        };

        let mut res = root_mod;
        let mut name = root_name;

        while let Some(sub_mod_name) = mod_parts.next() {
            res = res.get_field(sub_mod_name);
            name = sub_mod_name;
        }

        Ok((name.into(), res))
    }
    fn execute_stmt(&mut self, stmt: &AstNode) -> RuntimeResult<()> {
        match &stmt.kind {
            AstKind::VariableDefinition(name, value) => {
                let value = self.eval(&value)?;
                self.get_scope_mut().set(name.clone(), value)
            }
            AstKind::ArrayVariableDefinition(names, value) => {
                let value = self.eval(&value)?;
                let arr_ref = array!(value)?;
                let arr = arr_ref.lock().unwrap();
                let scope = self.get_scope_mut();
                for (i, name) in names.iter().enumerate() {
                    scope.set(name.clone(), arr.get(i).cloned().unwrap_or_default())
                }
            }
            AstKind::Documentation(_) => {}
            AstKind::Comment(_) => {}
            AstKind::VariableAssignment(name, value) => {
                let value = self.eval(&value)?;
                self.assign_variable(name.clone(), value)
            }
            AstKind::FunctionCall(name, arg_values) => match self.find(&name).clone() {
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
                    )?;
                }
                Dynamic::RustFunction { callback, .. } => {
                    let mut args = Vec::new();
                    for res in arg_values.iter().map(|a| self.eval(a)) {
                        match res {
                            Ok(value) => args.push(value),
                            Err(e) => return Err(e),
                        };
                    }
                    callback(self, args).unwrap_or_default();
                }
                actual => panic!("Expected {} to be a function, but it is a {}", name, actual),
            },
            AstKind::FunctionDefinition(name, args, body) => {
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
            AstKind::IfStatement(branches) => {
                for (cond, block) in branches {
                    if self.eval(&cond)?.is_true() {
                        self.scopes.push(Scope::default());
                        self.execute_stmts(&block)?;
                        self.scopes.pop();
                        break;
                    }
                }
            }
            AstKind::WhileStatement(cond, block) => {
                while !self.broken && self.eval(&cond)?.is_true() {
                    self.scopes.push(Scope::default());
                    self.execute_stmts(&block)?;
                    self.scopes.pop();
                    self.continued = false;
                }
                self.broken = false;
            }
            AstKind::ClassDefinition(name, members) => {
                let mut class = Class::new(&name);

                for member in members {
                    match member {
                        ClassMember::StaticFunction(name, arg_names, body) => {
                            let body = body.clone();
                            let arg_names = arg_names.clone();
                            class = class.add_static_function(&name, move |interp, args| {
                                //TODO: also capture scope
                                interp.call_fn(None, None, &arg_names, &args, &body)
                            });
                        }
                        ClassMember::Function(name, arg_names, body) => {
                            let body = body.clone();
                            let arg_names = arg_names.clone();
                            class = class.add_function(&name, move |interp, this, args| {
                                //TODO: also capture scope
                                interp.call_fn(Some(this), None, &arg_names, &args, &body)
                            });
                        }
                        ClassMember::Field(name, default) => {
                            class = class.add_field(&name, default.clone());
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
                                    interp.execute_stmts(&body)?;
                                    let result = interp.consume_return_value();
                                    interp.scopes.pop();
                                    Ok(result)
                                });
                        }
                    }
                }
            }
            AstKind::PlusAssignment(name, expr) => {
                let new_value = self.eval(&Expression::new(
                    ExpressionKind::BinaryOp(
                        Box::new(Expression::new(
                            ExpressionKind::Identifier(name.clone()),
                            stmt.location.clone(),
                        )),
                        Operator::Add,
                        Box::new(expr.clone()),
                    ),
                    stmt.location.clone(),
                ))?;
                self.assign_variable(name.clone(), new_value);
            }
            AstKind::MinusAssignment(name, expr) => {
                let new_value = self.eval(&Expression::new(
                    ExpressionKind::BinaryOp(
                        Box::new(Expression::new(
                            ExpressionKind::Identifier(name.clone()),
                            stmt.location.clone(),
                        )),
                        Operator::Subtract,
                        Box::new(expr.clone()),
                    ),
                    stmt.location.clone(),
                ))?;
                self.assign_variable(name.clone(), new_value);
            }
            AstKind::ReturnStatement(expr) => {
                self.return_value = Some(self.eval(expr)?);
            }
            AstKind::TimesAssignment(name, expr) => {
                let new_value = self.eval(&Expression::new(
                    ExpressionKind::BinaryOp(
                        Box::new(Expression::new(
                            ExpressionKind::Identifier(name.clone()),
                            stmt.location.clone(),
                        )),
                        Operator::Times,
                        Box::new(expr.clone()),
                    ),
                    stmt.location.clone(),
                ))?;
                self.assign_variable(name.clone(), new_value);
            }
            AstKind::DivideAssignment(name, expr) => {
                let new_value = self.eval(&Expression::new(
                    ExpressionKind::BinaryOp(
                        Box::new(Expression::new(
                            ExpressionKind::Identifier(name.clone()),
                            stmt.location.clone(),
                        )),
                        Operator::Divide,
                        Box::new(expr.clone()),
                    ),
                    stmt.location.clone(),
                ))?;
                self.assign_variable(name.clone(), new_value);
            }
            AstKind::BreakStatement => {
                self.broken = true;
            }
            AstKind::ContinueStatement => {
                self.continued = true;
            }
            AstKind::Expression(expr) => {
                self.eval(&expr)?;
            }
            AstKind::StaticFunctionDefinition(_, _, _) => unreachable!(),
            AstKind::ImportStatement(path) => {
                let (mod_name, module) = self.import(&path)?;
                self.get_scope_mut().set(mod_name, module);
            }
            AstKind::ExportStatement(ast) => {
                match &ast.kind {
                    AstKind::Expression(expr) => match &expr.kind {
                        ExpressionKind::Identifier(x) => {
                            self.exported_variables.push(x.into());
                        }
                        ExpressionKind::ClassIdentifier(x) => {
                            self.exported_classes.push(x.into());
                        }
                        _ => unreachable!(),
                    },
                    AstKind::VariableDefinition(name, _) => {
                        self.exported_variables.push(name.clone());
                        self.execute_stmt(&ast)?;
                    }
                    AstKind::FunctionDefinition(name, _, _) => {
                        self.exported_variables.push(name.clone());
                        self.execute_stmt(&ast)?;
                    }
                    AstKind::ClassDefinition(name, _) => {
                        self.exported_classes.push(name.clone());
                        self.execute_stmt(&ast)?;
                    }
                    _ => todo!(),
                };
            }
            _ => todo!("{:?}", stmt),
        }

        Ok(())
    }

    fn execute_stmts(&mut self, stmts: &Vec<AstNode>) -> RuntimeResult<()> {
        for stmt in stmts {
            self.execute_stmt(stmt)?;
            if self.return_value.is_some() || self.broken || self.continued {
                break;
            }
        }

        Ok(())
    }

    pub fn execute_file(&mut self, path: PathBuf) -> Result<(), String> {
        let mut parser = Parser::new();

        let content = std::fs::read_to_string(&path).unwrap();

        parser.set_source(path, &content, 0);

        let program = parser.parse()?;

        if self.debug {
            program.print();
        }

        self.execute(&program)?;

        Ok(())
    }

    pub fn execute(&mut self, prog: &Program) -> Result<Module, String> {
        let now = Instant::now();
        self.stmts = prog.stmts.clone();
        self.file_path = prog.path.clone();
        self.source = prog.source.to_string();
        self.execute_stmts(&prog.stmts)
            .map_err(|e| e.message(prog))?;

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

        if self.debug {
            let elapsed = now.elapsed();
            println!("Executing {:?} took {:?}", self.file_path, elapsed);
        }

        Ok(Module {
            name: "".into(),
            variables,
            scope: self.scopes.first().unwrap().clone(),
            functions,
            classes,
        })
    }
}

fn create_default_classes() -> HashMap<String, Class> {
    let mut classes = Vec::new();

    classes.push(
        Class::new("Number")
            .set_op_impl(Operator::Increment, |_, this, _| {
                number!(this).map(|x| x + 1)
            })
            .set_op_impl(Operator::Decrement, |_, this, _| {
                number!(this).map(|x| x - 1)
            })
            .add_static_function("from", |_, args| {
                Ok(match &args[0] {
                    Dynamic::String(x) => x.parse::<Number>().unwrap().into(),
                    _ => ().into(),
                })
            }),
    );
    classes.push(
        Class::new("String")
            .set_op_impl(Operator::Index, |_, this, args| {
                let this = string!(this)?;
                let idx = number!(args[0])?;

                Ok(this.chars().skip(idx as usize).next().unwrap_or_default())
            })
            .add_function("len", |_, this, _| {
                let this = string!(this)?;
                Ok(this.len() as Number)
            })
            .add_static_function("from", |_, args| {
                Ok(match &args[0] {
                    Dynamic::Boolean(x) => x.to_string().into(),
                    Dynamic::Number(x) => x.to_string().into(),
                    _ => Dynamic::Null,
                })
            })
            .add_function("split", |_, this, args| {
                let sep = string!(&args[0])?;
                let this = string!(this)?;
                Ok(this.split(sep).map(|x| x.into()).collect::<Vec<String>>())
            }),
    );
    classes.push(
        Class::new("Array")
            .add_function("push", |_, this, args| {
                let this_ref = array!(this)?;
                let mut this = this_ref.lock().unwrap();

                for arg in args {
                    this.push(arg.clone());
                }

                Ok(())
            })
            .add_function("len", |_, this, _| {
                let this_ref = array!(this)?;
                let this = this_ref.lock().unwrap();

                Ok(this.len() as Number)
            })
            .add_function("map", |i, this, args| {
                let items_ref = array!(this.clone())?;
                let items = items_ref.lock().unwrap();
                let cb = args[0].clone().as_fn().unwrap();

                let mut result = Vec::new();

                for item in items.iter() {
                    let mapped_item = cb.invoke(i, vec![item.clone()])?;
                    result.push(mapped_item);
                }

                Ok(result)
            })
            .add_function("fold", |i, this, args| {
                let items_ref = array!(this.clone())?;
                let items = items_ref.lock().unwrap();
                let initial = args[0].clone();
                let cb = args[1].clone().as_fn().unwrap();

                let mut acc = initial;

                for item in items.iter() {
                    acc = cb.invoke(i, vec![acc, item.clone()])?;
                }

                Ok(acc)
            })
            .add_function("filter", |i, this, args| {
                let items_ref = array!(this.clone())?;
                let items = items_ref.lock().unwrap();
                let cb = args[0].clone().as_fn().unwrap();

                let mut result = Vec::new();

                for item in items.iter().cloned() {
                    let passed = cb.invoke(i, vec![item.clone()])?.is_true();
                    if passed {
                        result.push(item);
                    }
                }

                Ok(result)
            })
            .add_function("contains", |i, this, args| {
                let items_ref = array!(this.clone())?;
                let items = items_ref.lock().unwrap();
                let value = &args[0];

                Ok(items.iter().find(|i| i == &value).is_some())
            })
            .add_function("for_each", |i, this, args| {
                let items_ref = array!(this.clone())?;
                let items = items_ref.lock().unwrap();
                let cb = args[0].clone().as_fn().unwrap();

                for item in items.iter() {
                    cb.invoke(i, vec![item.clone()])?;
                }

                Ok(())
            })
            .set_op_impl(Operator::Index, |_interp, this, args| {
                let this_ref = array!(this)?;
                let this = this_ref.lock().unwrap();
                let other = number!(args[0])?;

                Ok(this.get(other as usize).cloned().unwrap_or_default())
            }),
    );
    classes.push(Class::new("Null"));
    classes.push(Class::new("Module"));
    classes.push(
        Class::new("Class").set_op_impl(Operator::Constructor, |i, this, args| {
            let fields_ref = object!(&args[0])?;
            let fields = fields_ref.lock().unwrap();
            if let Dynamic::Class(this) = this {
                i.instantiate_class(&this.name, &fields)
            } else {
                unreachable!()
            }
        }),
    );
    classes.push(
        Class::new("Object")
            .set_op_impl(Operator::Index, |_, this, args| {
                let field = args[0].clone().as_str().unwrap();

                Ok(this.get_field(&field))
            })
            .add_function("keys", |_, this, _| {
                let this_ref = object!(this)?;
                let this = this_ref.lock().unwrap();

                Ok(this.keys().cloned().collect::<Vec<String>>())
            })
            .add_function("insert", |_, this, args| {
                let this_ref = object!(this)?;
                let mut this = this_ref.lock().unwrap();

                let key = args[0].clone();
                let value = args[1].clone();

                this.insert(key.to_string(), value);

                Ok(())
            })
            .add_function("remove", |_, this, args| {
                let this_ref = object!(this)?;
                let mut this = this_ref.lock().unwrap();

                let key = args[0].clone();

                this.remove(&key.to_string());

                Ok(())
            }),
    );
    classes.push(Class::new("Boolean"));
    classes.push(Class::new("Result"));
    classes.push(
        Class::new("Function").set_op_impl(Operator::Call, |i, this, args| {
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
        Class::new("RustFunction").set_op_impl(Operator::Call, |i, this, args| {
            if let Dynamic::RustFunction { callback, .. } = this {
                Ok(callback(i, args).unwrap_or_default())
            } else {
                unreachable!();
            }
        }),
    );

    classes.into_iter().map(|c| (c.name.clone(), c)).collect()
}

pub fn create_default_modules() -> HashMap<String, Module> {
    let mut map = HashMap::new();

    map.insert(
        "std".into(),
        Module::new("std").variable(
            "fs",
            Module::new("fs").function("read_file", |i, args| {
                let mut cwd = std::env::current_dir().unwrap();
                let rel_path = string!(&args[0])?;
                cwd.push(rel_path);
                Ok(std::fs::read_to_string(cwd).unwrap())
            }),
        ),
    );

    map
}

pub fn create_default_variables() -> HashMap<String, Dynamic> {
    ObjectBuilder::new()
        .function("print", |_, args| {
            println!("{}", args.iter().join(" "));

            Ok(())
        })
        .function("typeof", |_, args| Ok(args[0].type_name()))
        .function("require", |i, args| {
            let (_, module) = i.import(&args[0].clone().as_str().unwrap())?;
            Ok(module)
        })
        .function("atomic", |_, args| {
            let inner = args[0].clone();
            let mut wrapper = HashMap::new();
            wrapper.insert("value".into(), inner);
            Ok(Dynamic::new_object(wrapper))
        })
        .function("range", |_, args| match args.len() {
            1 => {
                let count = number!(args[0])?;
                let mut items = Vec::new();
                for i in 0..count {
                    items.push(Dynamic::Number(i));
                }
                Ok(Dynamic::new_array(items))
            }
            2 => {
                let start = number!(args[0])?;
                let count = number!(args[1])?;
                let mut items = Vec::new();
                for i in start..start + count {
                    items.push(Dynamic::Number(i));
                }
                Ok(Dynamic::new_array(items))
            }
            _ => todo!(),
        })
        .build()
}
