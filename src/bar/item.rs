use super::component::{Component, ComponentText};

#[derive(Debug, Clone)]
pub struct Item {
    pub left: i32,
    pub right: i32,
    pub component: Component,
    /// Vec of (left, right) and componenttext
    pub cached_result: Vec<((i32, i32), ComponentText)>,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            component: Component::default(),
            cached_result: Vec::new(),
            left: 0,
            right: 0,
        }
    }
}
