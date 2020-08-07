use crate::{split_direction::SplitDirection, window::Window};
use std::fmt::Debug;

#[derive(Clone)]
pub struct Tile {
    pub column: Option<i32>,
    pub row: Option<i32>,
    pub split_direction: SplitDirection,
    pub window: Window,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            column: None,
            row: None,
            split_direction: SplitDirection::Vertical,
            window: Window::default(),
        }
    }
}

impl Debug for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Tile(id: {}, title: '{}', row: {:?} column: {:?})",
            self.window.id, self.window.title, self.row, self.column
        ))
    }
}
