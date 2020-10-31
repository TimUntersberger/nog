use std::{collections::HashMap, path::PathBuf, fs};

use log::debug;

#[derive(Default, Debug, Clone)]
pub struct PluginManager {
    pub plugins: HashMap<String, String>,
}

impl PluginManager {
    pub fn load(&mut self) {
        let mut config_path: PathBuf = dirs::config_dir().unwrap_or_default();

        config_path.push("nog");
        if !config_path.exists() {
            debug!("nog folder doesn't exist yet. Creating the folder");
            fs::create_dir(config_path.clone()).map_err(|e| e.to_string());
        }        

        config_path.push("plugins");
        if !config_path.exists() {
            fs::create_dir(config_path.clone()).map_err(|e| e.to_string());
        }
        config_path.pop();

        config_path.push("plugins.json");
        if config_path.exists() {
            let raw_config = fs::read_to_string(config_path.clone()).unwrap();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw_config) {
                if let Some(config) = json.as_object() {
                    for (key, value) in config {
                        if let Some(s) = value.as_str() {
                            self.plugins.insert(key.clone(), s.to_string());
                        }
                    }
                }
            }
        }

        dbg!(&self.plugins);
    }
}
