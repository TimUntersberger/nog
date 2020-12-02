use super::{functions, lib, modules, syntax, types};
use crate::{config::Config, event::EventChannel, AppState};
use interpreter::{Dynamic, Function, Interpreter, Module};
use lazy_static::lazy_static;
use log::{debug, error};
use parking_lot::Mutex;
use rhai::{
    module_resolvers::{FileModuleResolver, ModuleResolversCollection},
    Engine, FnPtr, Scope,
};
use std::{io::Write, path::PathBuf, sync::Arc, thread};

lazy_static! {
    pub static ref MODE: Mutex<Option<String>> = Mutex::new(None);
    pub static ref ENGINE: Mutex<Engine> = Mutex::new(Engine::new());
    pub static ref SCOPE: Mutex<Scope<'static>> = Mutex::new(Scope::new());
    pub static ref AST: Mutex<rhai::AST> = Mutex::new(rhai::AST::default());
    pub static ref CALLBACKS: Mutex<Vec<FnPtr>> = Mutex::new(Vec::new());
}

pub fn add_callback(fp: FnPtr) -> usize {
    todo!()
}

pub fn call(idx: usize) {
    todo!()
}

fn build_relative_resolver(config_path: &PathBuf) -> FileModuleResolver {
    FileModuleResolver::new_with_path_and_extension(config_path.clone(), "nog")
}

pub fn parse_config(state_arc: Arc<Mutex<AppState>>) -> Result<Config, String> {
    todo!();
    // let mut engine = Engine::new();
    // let mut scope = Scope::new();
    // let config = Arc::new(Mutex::new(Config::default()));
    // let mut interpreter = Interpreter::new();
    // let cfg = config.clone();
    // interpreter.add_module(
    //     Module::new("nog")
    //         .variable("test", 2)
    //         .function("map", move |_i, args| {
    //             use std::str::FromStr;
    //             let mut kb = crate::keybindings::keybinding::Keybinding::from_str(&args[0].clone().as_str().unwrap()).unwrap();
    //             match &args[1] {
    //                 Dynamic::Function { body, scope, arg_names, name }  => {
    //                     let value = Function::new(&name, Some(scope), |i, args| {
    //                         i.call_fn(None, Some(scope), arg_names, args, body)
    //                     });
    //                     kb.typ = crate::keybindings::keybinding_type::KeybindingType::Callback(add_callback(value));
    //                 },
    //                 _ => todo!()
    //             }
    //             cfg.lock().add_keybinding(kb);
    //         }),
    // );

    // let mut config_path: PathBuf = dirs::config_dir().unwrap_or_default();

    // config_path.push("nog");

    // if !config_path.exists() {
    //     debug!("nog folder doesn't exist yet. Creating the folder");
    //     std::fs::create_dir(config_path.clone()).map_err(|e| e.to_string())?;
    // }

    // config_path.push("config.ns");

    // if !config_path.exists() {
    //     debug!("config file doesn't exist yet. Creating the file");
    //     if let Ok(mut file) = std::fs::File::create(config_path.clone()) {
    //         debug!("Initializing config with default values");
    //         // file.write_all(include_bytes!("../../../assets/default_config.nog"))
    //         //     .map_err(|e| e.to_string())?;
    //     }
    // }

    // debug!("Running config file");

    // interpreter.execute_file(config_path).unwrap();

    // *INTERPRETER.lock() = interpreter;
    //     syntax::init(&mut engine, state_arc.clone(), &mut config).unwrap();
    //     types::init(&mut engine);
    //     functions::init(&mut engine);
    //     lib::init(&mut engine, state_arc.clone());

    //     *CALLBACKS.lock() = Vec::new();

    //     let mut resolver_collection = ModuleResolversCollection::new();

    //     let modules_resolver = modules::new();
    //     resolver_collection.push(modules_resolver);

    //     let mut config_path: PathBuf = dirs::config_dir().unwrap_or_default();

    //     config_path.push("nog");

    //     let relative_resolver = build_relative_resolver(&config_path);

    //     resolver_collection.push(relative_resolver);

    //     engine.set_module_resolver(Some(resolver_collection));
    //     engine.set_max_expr_depths(0, 0);

    //     if !config_path.exists() {
    //         debug!("nog folder doesn't exist yet. Creating the folder");
    //         std::fs::create_dir(config_path.clone()).map_err(|e| e.to_string())?;
    //     }

    //     config_path.push("config.nog");

    //     if !config_path.exists() {
    //         debug!("config file doesn't exist yet. Creating the file");
    //         if let Ok(mut file) = std::fs::File::create(config_path.clone()) {
    //             debug!("Initializing config with default values");
    //             // file.write_all(include_bytes!("../../../assets/default_config.nog"))
    //             //     .map_err(|e| e.to_string())?;
    //         }
    //     }

    //     debug!("Parsing config file");
    //     let ast = engine
    //         .compile_file_with_scope(&scope, config_path)
    //         .map_err(|e| e.to_string())?;

    //     debug!("Running config file");
    //     engine
    //         .consume_ast_with_scope(&mut scope, &ast)
    //         .map_err(|e| e.to_string())?;

    //     *ENGINE.lock() = engine;
    //     *SCOPE.lock() = scope;
    //     *AST.lock() = ast;

    //     let mut config = config.lock().clone();

    //     dbg!(&config.keybindings);

    //     #[cfg(debug_assertions)]
    //     {
    //         config.work_mode = true;
    //     }

    //     Ok(config)
}
