use crate::keybindings::keybinding::Keybinding;
use log::error;
use regex::Regex;
use std::collections::HashMap;

pub mod hot_reloading;
pub mod rhai;

#[derive(Debug, Clone)]
pub struct Rule {
    pub pattern: Regex,
    pub has_custom_titlebar: bool,
    pub manage: bool,
    pub chromium: bool,
    pub firefox: bool,
    pub workspace_id: i32,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            pattern: Regex::new("").unwrap(),
            has_custom_titlebar: false,
            manage: true,
            chromium: false,
            firefox: false,
            workspace_id: -1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceSetting {
    pub id: i32,
    pub monitor: i32,
}

impl Default for WorkspaceSetting {
    fn default() -> Self {
        Self {
            id: -1,
            monitor: -1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub app_bar_height: i32,
    pub app_bar_bg: i32,
    pub app_bar_font: String,
    pub app_bar_date_pattern: String,
    pub app_bar_time_pattern: String,
    pub use_border: bool,
    pub app_bar_font_size: i32,
    pub min_width: i32,
    pub min_height: i32,
    pub work_mode: bool,
    pub light_theme: bool,
    pub multi_monitor: bool,
    pub launch_on_startup: bool,
    pub margin: i32,
    pub padding: i32,
    pub remove_title_bar: bool,
    pub remove_task_bar: bool,
    pub display_app_bar: bool,
    pub workspace_settings: Vec<WorkspaceSetting>,
    pub keybindings: Vec<Keybinding>,
    pub rules: Vec<Rule>,
    /// contains the metadata for each mode (like an icon)
    /// HashMap<mode, (Option<char>)>
    pub mode_meta: HashMap<String, (Option<char>)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_bar_height: 20,
            app_bar_bg: 0x2e3440,
            app_bar_font: String::from("Consolas"),
            app_bar_font_size: 18,
            app_bar_date_pattern: String::from("%e %b %Y"),
            app_bar_time_pattern: String::from("%T"),
            launch_on_startup: false,
            min_height: 0,
            min_width: 0,
            use_border: false,
            margin: 0,
            padding: 0,
            remove_title_bar: false,
            work_mode: true,
            light_theme: false,
            multi_monitor: false,
            remove_task_bar: false,
            display_app_bar: false,
            mode_meta: HashMap::new(),
            workspace_settings: Vec::new(),
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

    pub fn increment_field(self: &mut Self, field: &str, value: i32) {
        self.alter_numerical_field(field, value);
    }

    pub fn decrement_field(self: &mut Self, field: &str, value: i32) {
        self.alter_numerical_field(field, -value);
    }

    fn alter_numerical_field(self: &mut Self, field: &str, value: i32) {
        match field {
            "app_bar_height" => self.app_bar_height += value,
            "app_bar_bg" => self.app_bar_bg += value,
            "app_bar_font_size" => self.app_bar_font_size += value,
            "margin" => self.margin += value,
            "padding" => self.padding += value,
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
}
