use crate::bar::component::{self, Component};

#[derive(Clone, Debug)]
pub struct BarComponentsConfig {
    pub left: Vec<Component>,
    pub center: Vec<Component>,
    pub right: Vec<Component>,
}

impl Default for BarComponentsConfig {
    fn default() -> Self {
        Self {
            left: vec![component::workspaces::create()],
            center: vec![component::time::create("%T".into())],
            right: vec![
                component::active_mode::create(),
                component::padding::create(5),
                component::date::create("%e %b %Y".into()),
                component::padding::create(1),
            ],
        }
    }
}

impl BarComponentsConfig {
    pub fn empty(&mut self) {
        self.left = Vec::new();
        self.center = Vec::new();
        self.right = Vec::new();
    }
}

#[derive(Clone, Debug)]
pub struct BarConfig {
    pub height: i32,
    pub color: i32,
    pub font: String,
    pub font_size: i32,
    pub components: BarComponentsConfig,
}

impl PartialEq for BarConfig {
    fn eq(&self, other: &Self) -> bool {
        self.height == other.height
            && self.color == other.color
            && self.font == other.font
            && self.font_size == other.font_size
    }
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            height: 20,
            color: 0x2e3440,
            font: "Consolas".into(),
            font_size: 18,
            components: BarComponentsConfig::default(),
        }
    }
}
