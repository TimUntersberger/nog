use crate::hot_key_manager::{key::Key, modifier::Modifier, Direction, Keybinding, KeybindingType};
use crate::tile_grid::SplitDirection;
use log::debug;
use regex::Regex;
use std::io::{Error, ErrorKind, Write};
use std::str::FromStr;
use winapi::um::wingdi::GetBValue;
use winapi::um::wingdi::GetGValue;
use winapi::um::wingdi::GetRValue;
use winapi::um::wingdi::RGB;

#[macro_use]
mod macros;

pub mod hot_reloading;

#[derive(Debug, Clone)]
pub struct Rule {
    pub pattern: Regex,
    pub has_custom_titlebar: bool,
    pub manage: bool,
    pub chromium: bool,
    pub firefox: bool,
    pub remove_frame: bool,
    pub workspace: i32,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            pattern: Regex::new("").unwrap(),
            has_custom_titlebar: false,
            manage: true,
            remove_frame: true,
            chromium: false,
            firefox: false,
            workspace: -1,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub app_bar_height: i32,
    pub app_bar_bg: i32,
    pub app_bar_font: String,
    pub app_bar_font_size: i32,
    pub app_bar_workspace_bg: i32,
    pub work_mode: bool,
    pub launch_on_startup: bool,
    pub margin: i32,
    pub padding: i32,
    pub remove_title_bar: bool,
    pub remove_task_bar: bool,
    pub display_app_bar: bool,
    pub keybindings: Vec<Keybinding>,
    pub rules: Vec<Rule>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_bar_height: 20,
            app_bar_bg: 0x2c2427,
            app_bar_font: String::from("Consolas"),
            app_bar_font_size: 18,
            app_bar_workspace_bg: 0x161616,
            launch_on_startup: false,
            margin: 0,
            padding: 0,
            remove_title_bar: false,
            work_mode: true,
            remove_task_bar: false,
            display_app_bar: false,
            keybindings: Vec::new(),
            rules: Vec::new(),
        }
    }
}

impl Config {
    /// Creates a new default config.
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    let mut pathbuf = match dirs::config_dir() {
        Some(path) => path,
        None => std::path::PathBuf::new(),
    };

    pathbuf.push("wwm");

    if !pathbuf.exists() {
        debug!("wwm folder doesn't exist yet. Creating the folder");
        std::fs::create_dir(pathbuf.clone())?;
    }

    pathbuf.push("config.yaml");

    if !pathbuf.exists() {
        debug!("config file doesn't exist yet. Creating the file");
        if let Ok(mut file) = std::fs::File::create(pathbuf.clone()) {
            debug!("Initializing config with default values");
            file.write_all(include_bytes!("../default_config.yaml"))?;
        }
    }

    let path = match pathbuf.to_str() {
        Some(string) => string,
        None => "",
    };

    let file_content = std::fs::read_to_string(path)?;

    let vec_yaml = yaml_rust::YamlLoader::load_from_str(&file_content)?;
    let yaml = if !vec_yaml.is_empty() {
        &vec_yaml[0]
    } else {
        &yaml_rust::Yaml::Null
    };

    let mut config = Config::new();

    if let yaml_rust::yaml::Yaml::Hash(hash) = yaml {
        for entry in hash.iter() {
            let (key, value) = entry;
            let config_key = key.as_str().ok_or("Invalid config key")?;

            if_str!(config, config_key, value, app_bar_font);
            if_i32!(config, config_key, value, app_bar_bg);
            if_i32!(config, config_key, value, app_bar_workspace_bg);
            if_i32!(config, config_key, value, app_bar_font_size);
            if_i32!(config, config_key, value, app_bar_height);
            if_i32!(config, config_key, value, margin);
            if_i32!(config, config_key, value, padding);
            if_bool!(config, config_key, value, launch_on_startup);
            if_bool!(config, config_key, value, work_mode);
            if_bool!(config, config_key, value, remove_title_bar);
            if_bool!(config, config_key, value, remove_task_bar);
            if_bool!(config, config_key, value, display_app_bar);

            if config_key == "rules" {
                let rules = value.as_vec().ok_or("rules has to be an array")?;

                for yaml_rule in rules {
                    if let yaml_rust::Yaml::Hash(hash) = yaml_rule {
                        let mut rule = Rule::default();

                        for entry in hash.iter() {
                            let (key, value) = entry;
                            let hash_key = key.as_str().ok_or("Invalid config key")?;

                            if_regex!(rule, hash_key, value, pattern);
                            if_bool!(rule, hash_key, value, has_custom_titlebar);
                            if_bool!(rule, hash_key, value, remove_frame);
                            if_bool!(rule, hash_key, value, manage);
                            if_bool!(rule, hash_key, value, chromium);
                            if_bool!(rule, hash_key, value, firefox);
                            if_i32!(rule, hash_key, value, workspace);
                        }

                        config.rules.push(rule);
                    }
                }
            }

            if config_key == "keybindings" {
                let bindings = value.as_vec().ok_or("keybindings has to be an array")?;

                for binding in bindings {
                    let typ_str = ensure_str!("keybinding", binding, type);
                    let key_combo = ensure_str!("keybinding", binding, key);
                    let key_combo_parts = key_combo.split('+').collect::<Vec<&str>>();
                    let modifier_count = key_combo_parts.len() - 1;

                    let modifier = key_combo_parts
                        .iter()
                        .take(modifier_count)
                        .map(|x| match *x {
                            "Alt" => Modifier::ALT,
                            "Control" => Modifier::CONTROL,
                            "Shift" => Modifier::SHIFT,
                            _ => Modifier::default(),
                        })
                        .fold(Modifier::default(), |mut sum, crr| {
                            sum.insert(crr);

                            sum
                        });

                    let key = key_combo_parts
                        .iter()
                        .last()
                        .and_then(|x| Key::from_str(x).ok())
                        .ok_or("Invalid key")?;

                    let typ =
                        match typ_str {
                            "Launch" => KeybindingType::Launch(
                                ensure_str!("keybinding of type Launch", binding, cmd).to_string(),
                            ),
                            "CloseTile" => KeybindingType::CloseTile,
                            "Quit" => KeybindingType::Quit,
                            "ChangeWorkspace" => KeybindingType::ChangeWorkspace(ensure_i32!(
                                "keybinding of type ChangeWorkspace",
                                binding,
                                id
                            )),
                            "MoveToWorkspace" => KeybindingType::MoveToWorkspace(ensure_i32!(
                                "keybinding of type MoveToWorkspace",
                                binding,
                                id
                            )),
                            "ToggleFloatingMode" => KeybindingType::ToggleFloatingMode,
                            "ToggleWorkMode" => KeybindingType::ToggleWorkMode,
                            "Focus" => KeybindingType::Focus(Direction::from_str(ensure_str!(
                                "keybinding of type Focus",
                                binding,
                                direction
                            ))?),
                            "Swap" => KeybindingType::Swap(Direction::from_str(ensure_str!(
                                "keybinding of type Swap",
                                binding,
                                direction
                            ))?),
                            "Split" => KeybindingType::Split(SplitDirection::from_str(
                                ensure_str!("keybinding of type Split", binding, direction),
                            )?),
                            x => {
                                return Err(Box::new(Error::new(
                                    ErrorKind::InvalidInput,
                                    "unknown type ".to_string() + x,
                                )))
                            }
                        };

                    config.keybindings.push(Keybinding {
                        key,
                        modifier,
                        typ,
                        registered: false,
                    });
                }
            }
        }
        //Convert normal hexadecimal color format to winapi hexadecimal color format
        convert_color_format!(config.app_bar_bg);
        convert_color_format!(config.app_bar_workspace_bg);
    }
    Ok(config)
}
