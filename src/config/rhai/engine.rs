use super::{functions, syntax, modules};
use crate::{
    config::{Config, Rule, WorkspaceSetting, update_channel::UpdateChannel},
    keybindings::keybinding::Keybinding,
};
use log::{debug, error};
use rhai::{Array, Engine, Map, Scope};
use std::{io::Write, path::PathBuf, time::Duration};
use winapi::um::wingdi::{GetBValue, GetGValue, GetRValue, RGB};

macro_rules! set {
    ($typ: ty, $config: ident, $prop: ident, $key: ident, $val: ident) => {{
        if $key == stringify!($prop) {
            if $val.type_name().to_uppercase() != stringify!($typ).to_uppercase() {
                return Err(format!(
                    "{} has to be of type {} not {}",
                    stringify!($key),
                    stringify!($typ),
                    $val.type_name()
                ));
            } else {
                $config.$prop = $val.clone().cast::<$typ>();
                continue;
            }
        }
    }};
}

pub fn parse_config() -> Result<Config, String> {
    let mut engine = Engine::new();
    let mut scope = Scope::new();
    let mut config = Config::default();
    let modules_resolver = modules::new();

    engine.set_module_resolver(Some(modules_resolver));

    let mut config_path: PathBuf = dirs::config_dir().unwrap_or_default();

    config_path.push("wwm");

    if !config_path.exists() {
        debug!("wwm folder doesn't exist yet. Creating the folder");
        std::fs::create_dir(config_path.clone());
    }

    scope.set_value("__mode", None as Option<String>);
    scope.set_value("__cwd", config_path.to_str().unwrap().to_string());
    scope.set_value("__workspace_settings", Array::new());
    scope.set_value("__keybindings", Array::new());
    scope.set_value("__rules", Array::new());
    scope.set_value("__set", Map::new());
    scope.set_value("__update_channels", Array::new());

    functions::init(&mut engine);
    syntax::init(&mut engine).unwrap();

    config_path.push("config.rhai");

    if !config_path.exists() {
        debug!("config file doesn't exist yet. Creating the file");
        if let Ok(mut file) = std::fs::File::create(config_path.clone()) {
            debug!("Initializing config with default values");
            file.write_all(include_bytes!("../../../default_config.rhai"));
        }
    }

    debug!("Parsing config file");
    engine
        .consume_file_with_scope(&mut scope, config_path)
        .map_err(|e| e.to_string())?;

    let keybindings: Array = scope.get_value("__keybindings").unwrap();

    for val in keybindings {
        let boxed = val.cast::<Box<Keybinding>>();
        config.keybindings.push(*boxed);
    }

    let settings: Map = scope.get_value("__set").unwrap();

    for (key, value) in settings.iter().map(|(k, v)| (k.to_string(), v)) {
        set!(i32, config, min_height, key, value);
        set!(i32, config, min_width, key, value);
        set!(bool, config, launch_on_startup, key, value);
        set!(bool, config, multi_monitor, key, value);
        set!(bool, config, remove_title_bar, key, value);
        set!(bool, config, work_mode, key, value);
        set!(bool, config, remove_task_bar, key, value);
        set!(bool, config, display_app_bar, key, value);
        set!(bool, config, use_border, key, value);
        set!(bool, config, light_theme, key, value);
        set!(i32, config, margin, key, value);
        set!(i32, config, padding, key, value);
        set!(i32, config, app_bar_height, key, value);
        set!(String, config, app_bar_date_pattern, key, value);
        set!(String, config, app_bar_time_pattern, key, value);
        set!(String, config, app_bar_font, key, value);
        set!(i32, config, app_bar_font_size, key, value);
        set!(i32, config, app_bar_bg, key, value);
        if key == "update_interval" {
            if value.type_name().to_string() != "i32"  {
                return Err(format!(
                    "{} has to be of type {} not {}",
                    "update_interval",
                    "i32",
                    value.type_name()
                ));
            } else {
                config.update_interval = Duration::from_secs(value.clone().cast::<u64>() * 60);
                continue;
            }
        }
        if key == "default_update_channel" {
            if value.type_name().to_string() != "string"  {
                return Err(format!(
                    "{} has to be of type {} not {}",
                    "default_update_channel",
                    "String",
                    value.type_name()
                ));
            } else {
                config.default_update_channel = Some(value.clone().as_str().unwrap().to_string());
                continue;
            }
        }
        error!("Unknown setting {}", key);
    }

    config.app_bar_bg = RGB(
        GetBValue(config.app_bar_bg as u32),
        GetGValue(config.app_bar_bg as u32),
        GetRValue(config.app_bar_bg as u32),
    ) as i32;

    let rules: Array = scope.get_value("__rules").unwrap();

    for val in rules {
        let boxed = val.cast::<Box<Rule>>();
        config.rules.push(*boxed);
    }

    let workspace_settings: Array = scope.get_value("__workspace_settings").unwrap();

    for val in workspace_settings {
        let boxed = val.cast::<Box<WorkspaceSetting>>();
        config.workspace_settings.push(*boxed);
    }

    let update_channels: Array = scope.get_value("__update_channels").unwrap();

    for val in update_channels {
        let boxed = val.cast::<Box<UpdateChannel>>();
        config.update_channels.push(*boxed);
    }

    Ok(config)
}
