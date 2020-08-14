use super::item::Item;

#[derive(Debug, Clone)]
pub struct ItemSection {
    pub left: i32,
    pub right: i32,
    pub items: Vec<Item>,
}

impl ItemSection {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
}

impl Default for ItemSection {
    fn default() -> Self {
        Self {
            left: 0,
            right: 0,
            items: Vec::new(),
        }
    }
}
