use crate::keybindings::{
    key::Key, keybinding::Keybinding, keybinding_type::KeybindingType, modifier::Modifier,
};
use bar_config::BarConfig;
use log::error;
use rule::Rule;
use std::{collections::HashMap, time::Duration};
use update_channel::UpdateChannel;
use workspace_setting::WorkspaceSetting;

pub mod bar_config;
pub mod hot_reloading;
pub mod rhai;
pub mod rule;
pub mod update_channel;
pub mod workspace_setting;

#[derive(Clone, Debug)]
pub struct Config {
    pub use_border: bool,
    pub min_width: i32,
    pub min_height: i32,
    pub work_mode: bool,
    pub light_theme: bool,
    pub multi_monitor: bool,
    pub launch_on_startup: bool,
    pub outer_gap: i32,
    pub inner_gap: i32,
    pub remove_title_bar: bool,
    pub remove_task_bar: bool,
    pub ignore_fullscreen_actions: bool,
    pub display_app_bar: bool,
    pub bar: BarConfig,
    pub workspace_settings: Vec<WorkspaceSetting>,
    pub keybindings: Vec<Keybinding>,
    pub rules: Vec<Rule>,
    pub update_channels: Vec<UpdateChannel>,
    pub default_update_channel: Option<String>,
    pub update_interval: Duration,
    /// contains the metadata for each mode (like an icon)
    /// HashMap<mode, (Option<char>)>
    pub mode_meta: HashMap<String, Option<char>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            launch_on_startup: false,
            min_height: 200,
            min_width: 200,
            use_border: true,
            outer_gap: 0,
            inner_gap: 0,
            remove_title_bar: true,
            work_mode: true,
            light_theme: false,
            multi_monitor: false,
            remove_task_bar: true,
            display_app_bar: true,
            ignore_fullscreen_actions: false,
            bar: BarConfig::default(),
            mode_meta: HashMap::new(),
            workspace_settings: Vec::new(),
            keybindings: vec![Keybinding {
                typ: KeybindingType::CloseTile,
                mode: None,
                key: Key::Q,
                modifier: Modifier::ALT,
            }],
            rules: Vec::new(),
            update_channels: Vec::new(),
            default_update_channel: None,
            update_interval: Duration::from_secs(60 * 60),
        }
    }
}

impl Config {
    /// Creates a new default config.
    pub fn new() -> Self {
        let mut temp = Self::default();
        temp.keybindings = Vec::new();
        temp
    }

    pub fn increment_field(&self, field: &str, value: i32) -> Config {
        self.alter_numerical_field(field, value)
    }

    pub fn decrement_field(&self, field: &str, value: i32) -> Config {
        self.alter_numerical_field(field, -value)
    }

    fn alter_numerical_field(&self, field: &str, value: i32) -> Config {
        let mut config = self.clone();
        match field {
            "bar.height" => config.bar.height += value,
            "bar.color" => config.bar.color += value,
            "bar.font_size" => config.bar.font_size += value,
            "outer_gap" => config.outer_gap += value,
            "inner_gap" => config.inner_gap += value,
            _ => error!("Attempt to alter unknown field: {} by {}", field, value),
        }
        config
    }

    pub fn toggle_field(&self, field: &str) -> Config {
        let mut config = self.clone();
        match field {
            "use_border" => config.use_border = !config.use_border,
            "light_theme" => config.light_theme = !config.light_theme,
            "launch_on_startup" => config.launch_on_startup = !config.launch_on_startup,
            "remove_title_bar" => config.remove_title_bar = !config.remove_title_bar,
            "remove_task_bar" => config.remove_task_bar = !config.remove_task_bar,
            "ignore_fullscreen_actions" => config.ignore_fullscreen_actions = !config.ignore_fullscreen_actions,
            "display_app_bar" => config.display_app_bar = !config.display_app_bar,
            _ => error!("Attempt to toggle unknown field: {}", field),
        }
        config
    }

    pub fn add_keybinding(&mut self, keybinding: Keybinding) {
        if let Some(kb) = self
            .keybindings
            .iter_mut()
            .find(|kb| kb.key == keybinding.key && kb.modifier == keybinding.modifier && kb.mode == keybinding.mode)
        {
            kb.typ = keybinding.typ;
            kb.mode = keybinding.mode;
        } else {
            self.keybindings.push(keybinding);
        }
    }

    pub fn get_keybinding_of_type(&self, kind: KeybindingType) -> Option<&Keybinding> {
        self.keybindings.iter().find(|kb| kb.typ == kind)
    }

    pub fn set_bool_field(&self, field: &str, value: bool) -> Config {
        let mut config = self.clone();
        match field {
            "use_border" => config.use_border = value,
            "light_theme" => config.light_theme = value,
            "launch_on_startup" => config.launch_on_startup = value,
            "remove_title_bar" => config.remove_title_bar = value,
            "remove_task_bar" => config.remove_task_bar = value,
            "ignore_fullscreen_actions" => config.ignore_fullscreen_actions = value,
            "display_app_bar" => config.display_app_bar = value,
            _ => error!("Attempt to set unknown field: {}", field),
        }
        config
    }

    pub fn get_update_channel(&self) -> Option<&UpdateChannel> {
        self.default_update_channel
            .clone()
            .and_then(|name| self.update_channels.iter().find(|c| c.name == name))
    }
}
