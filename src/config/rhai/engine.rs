use super::{functions, modules, syntax};
use crate::{
    config::{update_channel::UpdateChannel, Config, Rule, WorkspaceSetting},
    keybindings::keybinding::Keybinding,
};
use lazy_static::lazy_static;
use log::debug;
use rhai::{
    module_resolvers::{FileModuleResolver, ModuleResolversCollection},
    Array, Engine, Map, Scope,
};
use std::{cell::RefCell, io::Write, path::PathBuf, rc::Rc, sync::Mutex};
use winapi::um::wingdi::{GetBValue, GetGValue, GetRValue, RGB};

lazy_static! {
    pub static ref MODE: Mutex<Option<String>> = Mutex::new(None);
}

pub fn parse_config() -> Result<Config, String> {
    let mut engine = Engine::new();
    let mut scope = Scope::new();
    let mut config = Rc::new(RefCell::new(Config::default()));
    let mut resolver_collection = ModuleResolversCollection::new();

    let modules_resolver = modules::new();
    resolver_collection.push(modules_resolver);

    let mut config_path: PathBuf = dirs::config_dir().unwrap_or_default();

    config_path.push("nog");

    let relative_resolver =
        FileModuleResolver::new_with_path_and_extension(config_path.clone(), "nog");
    resolver_collection.push(relative_resolver);

    engine.set_module_resolver(Some(resolver_collection));

    if !config_path.exists() {
        debug!("nog folder doesn't exist yet. Creating the folder");
        std::fs::create_dir(config_path.clone());
    }

    functions::init(&mut engine);
    syntax::init(&mut engine, &mut config).unwrap();

    config_path.push("config.nog");

    if !config_path.exists() {
        debug!("config file doesn't exist yet. Creating the file");
        if let Ok(mut file) = std::fs::File::create(config_path.clone()) {
            debug!("Initializing config with default values");
            file.write_all(include_bytes!("../../../assets/default_config.nog"));
        }
    }

    debug!("Parsing config file");
    engine
        .consume_file_with_scope(&mut scope, config_path)
        .map_err(|e| e.to_string())?;

    let mut config = config.borrow().clone();

    config.bar.color = RGB(
        GetBValue(config.bar.color as u32),
        GetGValue(config.bar.color as u32),
        GetRValue(config.bar.color as u32),
    ) as i32;

    Ok(config)
}
