use std::{collections::HashMap, fs, path::PathBuf, process::Command};

use log::debug;
use rhai::{module_resolvers::StaticModuleResolver, Engine, Module};

#[derive(Default, Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub path: PathBuf,
    pub url: String,
}

#[derive(Default, Debug, Clone)]
pub struct PluginManager {
    plugins_json_path: PathBuf,
    plugins_folder_path: PathBuf,
    pub plugins: Vec<Plugin>,
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

        self.plugins_folder_path = config_path.clone();

        if !config_path.exists() {
            fs::create_dir(config_path.clone()).map_err(|e| e.to_string());
        }

        config_path.pop();

        config_path.push("plugins.json");

        self.plugins_json_path = config_path.clone();

        if config_path.exists() {
            let raw_config = fs::read_to_string(config_path.clone()).unwrap();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw_config) {
                if let Some(config) = json.as_object() {
                    for (key, value) in config {
                        let mut path = self.plugins_folder_path.clone();
                        path.push(key.clone());
                        if let Some(s) = value.as_str() {
                            self.plugins.push(Plugin {
                                name: key.clone(),
                                url: s.to_string(),
                                path,
                            });
                        }
                    }
                }
            }
        }
    }

    fn install_plugin(&self, plugin: &Plugin) {
        Command::new("git")
            .arg("clone")
            .arg(format!("https://www.github.com/{}", plugin.url))
            .arg(&plugin.path)
            .spawn()
            .unwrap();
    }

    fn update_plugin(&self, plugin: &Plugin) {
        Command::new("git")
            .arg("pull")
            .arg(format!("https://www.github.com/{}", plugin.url))
            .arg(&plugin.path)
            .spawn()
            .unwrap();
    }

    pub fn install(&self) {
        for plugin in &self.plugins {
            if !plugin.path.exists() {
                self.install_plugin(plugin);
            }
        }
    }

    pub fn update(&self) {
        for plugin in &self.plugins {
            if !plugin.path.exists() {
                self.install_plugin(plugin);
            } else {
                self.update_plugin(plugin);
            }
        }
    }

    pub fn purge(&self) {
        if let Ok(list) = fs::read_dir(&self.plugins_folder_path) {
            for folder in list {
                if let Ok(folder) = folder {
                    if !self.plugins.iter().any(|p| &p.name == folder.file_name().to_str().unwrap()) {
                        fs::remove_dir(folder.path()).expect("Failed to purge");
                    }
                }
            }
        }
    }
}
