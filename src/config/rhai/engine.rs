use super::{functions, modules, syntax};
use crate::{
    DISPLAYS,
    config::{update_channel::UpdateChannel, Config, Rule, WorkspaceSetting},
    keybindings::keybinding::Keybinding,
    popup::Popup,
};
use lazy_static::lazy_static;
use log::debug;
use rhai::{
    module_resolvers::{FileModuleResolver, ModuleResolversCollection},
    Array, Engine, ImmutableString, Map, RegisterFn, Scope, Dynamic,
};
use std::{cell::RefCell, collections::HashMap, io::Write, path::PathBuf, rc::Rc, sync::Mutex};
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

    engine.register_fn(
        "popup_new",
        |name: ImmutableString, width: i32, height: i32, options: Map| {
            let mut p = Popup::new(&name, width, height);

            for (key, val) in options {
                match key.as_str() {
                    "text" => p.with_text(
                        val.cast::<Array>()
                            .iter()
                            .map(|v| v.as_str().unwrap())
                            .collect::<Vec<&str>>()
                            .as_slice(),
                    ),
                    "padding" => p.with_padding(val.as_int().unwrap()),
                    _ => &mut p
                };
            }

            std::thread::spawn(move || {
                loop {
                    std::thread::sleep_ms(20);
                    if DISPLAYS.lock().unwrap().len() > 0 {
                        break;
                    }
                }
                p.create();
            });
        },
    );

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
