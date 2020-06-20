use crate::hot_key_manager::{Key, Modifier};
use crate::tile_grid::SplitDirection;
use regex::Regex;
use std::io::{Error, ErrorKind};

#[macro_use]
mod macros;

use std::str::FromStr;

pub type Command = String;
//TODO(#19)
pub type FocusDirection = String;

pub enum Keybinding {
    CloseTile(Key, Vec<Modifier>),
    Quit(Key, Vec<Modifier>),
    ChangeWorkspace(Key, Vec<Modifier>, i32),
    ToggleFloatingMode(Key, Vec<Modifier>),
    Shell(Key, Vec<Modifier>, Command),
    Focus(Key, Vec<Modifier>, FocusDirection),
    Split(Key, Vec<Modifier>, SplitDirection),
}

#[derive(Debug)]
pub struct Rule {
    pub pattern: Regex,
    pub has_custom_titlebar: bool,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            pattern: Regex::new("").unwrap(),
            has_custom_titlebar: false,
        }
    }
}

pub struct Config {
    pub app_bar_bg: i32,
    pub app_bar_workspace_bg: i32,
    pub remove_title_bar: bool,
    pub remove_task_bar: bool,
    pub display_app_bar: bool,
    pub keybindings: Vec<Keybinding>,
    pub rules: Vec<Rule>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            app_bar_bg: 0x0027242c,
            app_bar_workspace_bg: 0x00161616,
            remove_title_bar: false,
            remove_task_bar: false,
            display_app_bar: false,
            keybindings: Vec::new(),
            rules: Vec::new(),
        }
    }
}

pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    let mut pathbuf = match dirs::config_dir() {
        Some(path) => path,
        None => std::path::PathBuf::new(),
    };

    pathbuf.push("wwm");

    if !pathbuf.exists() {
        std::fs::create_dir(pathbuf.clone())?;
    }

    pathbuf.push("config.yaml");

    if !pathbuf.exists() {
        std::fs::File::create(pathbuf.clone())?;
    }

    let path = match pathbuf.to_str() {
        Some(string) => string,
        None => "",
    };

    let file_content = std::fs::read_to_string(path).unwrap();

    let vec_yaml = yaml_rust::YamlLoader::load_from_str(&file_content).unwrap();
    let mut yaml = &yaml_rust::Yaml::Null;
    if !vec_yaml.is_empty() {
        yaml = &vec_yaml[0];
    }

    let mut config = Config::new();

    if let yaml_rust::yaml::Yaml::Hash(hash) = yaml {
        for entry in hash.iter() {
            let (key, value) = entry;
            let config_key = key.as_str().ok_or("Invalid config key")?;

            if_hex!(config, config_key, value, app_bar_bg);
            if_hex!(config, config_key, value, app_bar_workspace_bg);
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
                        }

                        config.rules.push(rule);
                    }
                }
            }

            if config_key == "keybindings" {
                let bindings = value.as_vec().ok_or("keybindings has to be an array")?;

                for binding in bindings {
                    //type
                    let typ = ensure_str!("keybinding", binding, type);
                    let key_combo = ensure_str!("keybinding", binding, key);
                    let key_combo_parts = key_combo.split("+").collect::<Vec<&str>>();
                    let modifier_count = key_combo_parts.len() - 1;

                    let modifiers = key_combo_parts
                        .iter()
                        .take(modifier_count)
                        .map(|x| Modifier::from_str(x).unwrap())
                        .collect::<Vec<Modifier>>();

                    let key = key_combo_parts
                        .iter()
                        .last()
                        .and_then(|x| Key::from_str(x).ok())
                        .ok_or("Invalid key")?;

                    let keybinding = match typ {
                        "Shell" => Keybinding::Shell(
                            key,
                            modifiers,
                            ensure_str!("keybinding of type shell", binding, cmd).to_string(),
                        ),
                        "CloseTile" => Keybinding::CloseTile(key, modifiers),
                        "Quit" => Keybinding::Quit(key, modifiers),
                        "ChangeWorkspace" => Keybinding::ChangeWorkspace(
                            key,
                            modifiers,
                            ensure_i32!("keybinding of type shell", binding, id),
                        ),
                        "ToggleFloatingMode" => Keybinding::ToggleFloatingMode(key, modifiers),
                        "Focus" => Keybinding::Focus(
                            key,
                            modifiers,
                            ensure_str!("keybinding of type shell", binding, direction).to_string(),
                        ),
                        "Split" => Keybinding::Split(
                            key,
                            modifiers,
                            SplitDirection::from_str(ensure_str!("keybinding of type shell", binding, direction))?
                        ),
                        x => {
                            return Err(Box::new(Error::new(
                                ErrorKind::InvalidInput,
                                "unknown type ".to_string() + x,
                            )))
                        }
                    };

                    config.keybindings.push(keybinding);
                }
            }
        }
    }
    Ok(config)
}
