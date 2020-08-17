use super::component::Component;

#[derive(Debug, Clone)]
pub struct Item {
    pub left: i32,
    pub right: i32,
    pub component: Component,
    pub widths: Vec<(i32, i32)>,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            widths: Vec::new(),
            component: Component::default(),
            left: 0,
            right: 0,
        }
    }
}
