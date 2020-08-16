use super::{functions, modules, syntax, lib};
use crate::{
    DISPLAYS,
    config::{update_channel::UpdateChannel, Config, Rule, WorkspaceSetting},
    keybindings::keybinding::Keybinding,
    popup::Popup,
};
use lazy_static::lazy_static;
use log::{error, debug};
use rhai::{
    module_resolvers::{FileModuleResolver, ModuleResolversCollection},
    Array, Engine, ImmutableString, Map, RegisterFn, Scope, Dynamic,
};
use std::{cell::RefCell, collections::HashMap, io::Write, path::PathBuf, rc::Rc, sync::{Arc, Mutex}};
use winapi::um::wingdi::{GetBValue, GetGValue, GetRValue, RGB};

lazy_static! {
    pub static ref MODE: Mutex<Option<String>> = Mutex::new(None);
    pub static ref ENGINE: Mutex<Engine> = Mutex::new(Engine::new());
    pub static ref SCOPE: Mutex<Scope<'static>> = Mutex::new(Scope::new());
    pub static ref AST: Mutex<rhai::AST> = Mutex::new(rhai::AST::default());
}

pub fn call(fn_name: &str) {
    let engine = ENGINE.lock().unwrap();
    let mut scope = SCOPE.lock().unwrap();
    let ast = AST.lock().unwrap();
    let _ = engine
        .call_fn::<(), ()>(&mut *scope, &*ast, fn_name, ())
        .map_err(|e| error!("{}", e.to_string()));
}

pub fn parse_config() -> Result<Config, String> {
    let mut engine = Engine::new();
    let mut scope = Scope::new();
    let mut config = Arc::new(Mutex::new(Config::default()));
    let mut resolver_collection = ModuleResolversCollection::new();

    let modules_resolver = modules::new();
    resolver_collection.push(modules_resolver);

    let mut config_path: PathBuf = dirs::config_dir().unwrap_or_default();

    config_path.push("nog");

    let relative_resolver =
        FileModuleResolver::new_with_path_and_extension(config_path.clone(), "nog");
    resolver_collection.push(relative_resolver);

    engine.set_module_resolver(Some(resolver_collection));
    engine.set_max_expr_depths(0, 0);

    if !config_path.exists() {
        debug!("nog folder doesn't exist yet. Creating the folder");
        std::fs::create_dir(config_path.clone());
    }

    config_path.push("config.nog");

    if !config_path.exists() {
        debug!("config file doesn't exist yet. Creating the file");
        if let Ok(mut file) = std::fs::File::create(config_path.clone()) {
            debug!("Initializing config with default values");
            file.write_all(include_bytes!("../../../assets/default_config.nog"));
        }
    }

    syntax::init(&mut engine, &mut config).unwrap();
    functions::init(&mut engine);
    lib::init(&mut engine);

    debug!("Parsing config file");
    let ast = engine.compile_file_with_scope(&mut scope, config_path)
        .map_err(|e| e.to_string())?;

    debug!("Running config file");
    engine
        .consume_ast_with_scope(&mut scope, &ast)
        .map_err(|e| e.to_string())?;

    *ENGINE.lock().unwrap() = engine;
    *SCOPE.lock().unwrap() = scope;
    *AST.lock().unwrap() = ast.clone();

    let mut config = config.lock().unwrap().clone();

    config.bar.color = RGB(
        GetBValue(config.bar.color as u32),
        GetGValue(config.bar.color as u32),
        GetRValue(config.bar.color as u32),
    ) as i32;

    Ok(config)
}
