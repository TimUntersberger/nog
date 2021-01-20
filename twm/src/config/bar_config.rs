use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    bar::component::{self, Component},
    AppState,
};

#[derive(Clone, Debug, Default)]
pub struct BarComponentsConfig {
    pub left: Vec<Component>,
    pub center: Vec<Component>,
    pub right: Vec<Component>,
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

impl BarConfig {
    pub fn use_default_components(&mut self, state_arc: Arc<Mutex<AppState>>) {
        self.components.left = vec![component::workspaces::create(state_arc.clone())];
        self.components.center = vec![component::time::create("%T".into())];
        self.components.right = vec![
            component::active_mode::create(state_arc.clone()),
            component::padding::create(5),
            component::split_direction::create(state_arc.clone(), "V".into(), "H".into()),
            component::padding::create(5),
            component::date::create("%e %b %Y".into()),
            component::padding::create(1),
        ];
    }
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
            color: 0x40342e,
            font: "Consolas".into(),
            font_size: 18,
            components: BarComponentsConfig::default(),
        }
    }
}
