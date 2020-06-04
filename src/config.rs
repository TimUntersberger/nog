use crate::hot_key_manager::{Modifier, Key};
use crate::tile_grid::SplitDirection;
use std::io::{Error, ErrorKind};

use std::str::FromStr;

pub type Command = String;
//TODO(#19)
pub type FocusDirection = String;

pub enum Keybinding {
    CloseTile(Key, Vec<Modifier>),
    Quit(Key, Vec<Modifier>),
    ToggleFloatingMode(Key, Vec<Modifier>),
    Shell(Key, Vec<Modifier>, Command),
    Focus(Key, Vec<Modifier>, FocusDirection),
    Split(Key, Vec<Modifier>, SplitDirection)
}

pub struct Config {
    pub remove_title_bar: bool,
    pub keybindings: Vec<Keybinding>
}

impl Config {
    pub fn new() -> Self {
        Self {
            remove_title_bar: true,
            keybindings: Vec::new()
        }
    }
}

pub fn load() -> Result<Config, Box<dyn std::error::Error>>{
    let mut pathbuf = match dirs::config_dir() {
        Some(path) => path,
        None => std::path::PathBuf::new()
    };

    pathbuf.push("wwm");

    if !pathbuf.exists() {
        std::fs::create_dir(pathbuf.clone());
    }

    pathbuf.push("config.yaml");

    if !pathbuf.exists() {
        std::fs::File::create(pathbuf.clone());
    }

    let path = match pathbuf.to_str() {
        Some(string) => string,
        None => ""
    };

    println!("Config path: {}", path);

    let file_content = std::fs::read_to_string(path).unwrap();
    let yaml = &yaml_rust::YamlLoader::load_from_str(&file_content).unwrap()[0];
    let mut config = Config::new();

    if let yaml_rust::yaml::Yaml::Hash(hash) = yaml {
        for entry in hash.iter() {
            let (key, value) = entry;
            let config_key = key.as_str().ok_or("Invalid config key")?;

            match config_key {
                "remove_title_bar" => {
                    config.remove_title_bar = value.as_bool().ok_or("remove_title_bar has to a bool")?;
                },
                "keybindings" => {
                    let bindings = value.as_vec().ok_or("keybindings has to be an array")?; 

                    for binding in bindings {
                        //type
                        let typ = binding["type"].as_str().ok_or("a keybinding has to have a type property")?;
                        let key_combo = binding["key"].as_str().ok_or("a keybinding has to have a key property")?;
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
                            "shell" => Keybinding::Shell(
                                key,
                                modifiers,
                                binding["cmd"]
                                    .as_str()
                                    .ok_or("a keybinding of type shell has to have a cmd property")?
                                    .to_string()
                            ),
                            "CloseTile" => Keybinding::CloseTile(key, modifiers),
                            "Quit" => Keybinding::Quit(key, modifiers),
                            "ToggleFloatingMode" => Keybinding::ToggleFloatingMode(key, modifiers),
                            "Focus" => Keybinding::Focus(
                                key, 
                                modifiers, 
                                binding["direction"]
                                    .as_str()
                                    .ok_or("a keybinding of type shell has to have a direction property")?
                                    .to_string()
                            ),
                            "Split" => Keybinding::Split(
                                key, 
                                modifiers, 
                                binding["direction"]
                                    .as_str()
                                    .ok_or("a keybinding of type shell has to have a direction property")
                                    .map(SplitDirection::from_str)?? // xd double question mark
                                ),
                            x => return Err(Box::new(Error::new(ErrorKind::InvalidInput, "unknown type ".to_string() + x)))
                        };

                        config.keybindings.push(keybinding);

                        println!("type: {} | key: {}", typ, key_combo);
                    }
                },
                s => {
                    return Err(Box::new(Error::new(ErrorKind::InvalidInput, "unknown option ".to_string() + s)));
                }
            }
        }
    }
    Ok(config)
}