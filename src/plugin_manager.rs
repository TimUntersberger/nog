use log::debug;
use std::{fs, path::PathBuf, process::Command};

#[derive(Default, Debug, Clone)]
pub struct Plugin {
    pub path: PathBuf,
    pub url: String,
}

impl Plugin {
    pub fn name(&self) -> &str {
        self.url.split("/").last().unwrap()
    }
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
                if let Some(config) = json.as_array() {
                    for value in config {
                        let mut path = self.plugins_folder_path.clone();
                        if let Some(s) = value.as_str() {
                            let mut plugin = Plugin {
                                url: s.to_string(),
                                path: Default::default(),
                            };
                            path.push(plugin.name());
                            plugin.path = path;
                            self.plugins.push(plugin);
                        }
                    }
                }
            }
        }
    }

    fn install_plugin(&self, plugin: &Plugin) {
        debug!("Installing {:?} from {}", plugin.name(), plugin.url);
        Command::new("git")
            .arg("clone")
            .arg(format!("https://www.github.com/{}", plugin.url))
            .arg(&plugin.path)
            .spawn()
            .unwrap()
            .wait();
    }

    fn update_plugin(&self, plugin: &Plugin) {
        debug!("Updating {:?}", plugin.name());
        Command::new("git")
            .current_dir(&plugin.path)
            .arg("pull")
            .spawn()
            .unwrap()
            .wait();
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
                    if !self
                        .plugins
                        .iter()
                        .any(|p| p.name() == folder.file_name().to_str().unwrap())
                    {
                        debug!("Purging {:?}", folder.file_name());
                        fs::remove_dir_all(folder.path()).expect("Failed to purge");
                    }
                }
            }
        }
    }
}
