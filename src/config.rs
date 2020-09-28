use crate::keybindings::keybinding::Keybinding;
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
    pub display_app_bar: bool,
    pub bar: BarConfig,
    pub workspace_settings: Vec<WorkspaceSetting>,
    pub keybindings: Vec<Keybinding>,
    pub rules: Vec<Rule>,
    pub update_channels: Vec<UpdateChannel>,
    pub default_update_channel: Option<String>,
    pub update_interval: Duration, //minutes
    /// contains the metadata for each mode (like an icon)
    /// HashMap<mode, (Option<char>)>
    pub mode_meta: HashMap<String, Option<char>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            launch_on_startup: false,
            min_height: 0,
            min_width: 0,
            use_border: false,
            outer_gap: 0,
            inner_gap: 0,
            remove_title_bar: false,
            work_mode: true,
            light_theme: false,
            multi_monitor: false,
            remove_task_bar: false,
            display_app_bar: false,
            bar: BarConfig::default(),
            mode_meta: HashMap::new(),
            workspace_settings: Vec::new(),
            keybindings: Vec::new(),
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
        Self::default()
    }

    pub fn increment_field(self: &mut Self, field: &str, value: i32) {
        self.alter_numerical_field(field, value);
    }

    pub fn decrement_field(self: &mut Self, field: &str, value: i32) {
        self.alter_numerical_field(field, -value);
    }

    fn alter_numerical_field(self: &mut Self, field: &str, value: i32) {
        match field {
            "bar.height" => self.bar.height += value,
            "bar.color" => self.bar.color += value as u32,
            "bar.font_size" => self.bar.font_size += value,
            "outer_gap" => self.outer_gap += value,
            "inner_gap" => self.inner_gap += value,
            _ => error!("Attempt to alter unknown field: {} by {}", field, value),
        }
    }

    pub fn toggle_field(self: &mut Self, field: &str) {
        match field {
            "use_border" => self.use_border = !self.use_border,
            "light_theme" => self.light_theme = !self.light_theme,
            "launch_on_startup" => self.launch_on_startup = !self.launch_on_startup,
            "remove_title_bar" => self.remove_title_bar = !self.remove_title_bar,
            "remove_task_bar" => self.remove_task_bar = !self.remove_task_bar,
            "display_app_bar" => self.display_app_bar = !self.display_app_bar,
            _ => error!("Attempt to toggle unknown field: {}", field),
        }
    }

    pub fn set_bool_field(self: &mut Self, field: &str, value: bool) {
        match field {
            "use_border" => self.use_border = value,
            "light_theme" => self.light_theme = value,
            "launch_on_startup" => self.launch_on_startup = value,
            "remove_title_bar" => self.remove_title_bar = value,
            "remove_task_bar" => self.remove_task_bar = value,
            "display_app_bar" => self.display_app_bar = value,
            _ => error!("Attempt to set unknown field: {}", field),
        }
    }

    pub fn get_update_channel(&self) -> Option<&UpdateChannel> {
        self.default_update_channel
            .clone()
            .and_then(|name| self.update_channels.iter().find(|c| c.name == name))
    }
}
