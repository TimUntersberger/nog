use crate::tile_grid::SplitDirection;
use crate::window::Window;
use std::fmt::Debug;

#[derive(Clone)]
pub struct Tile {
    pub column: Option<i32>,
    pub row: Option<i32>,
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
    pub split_direction: SplitDirection,
    pub window: Window,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            column: None,
            row: None,
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
            split_direction: SplitDirection::Vertical,
            window: Window::default(),
        }
    }
}

impl Debug for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Tile(id: {}, title: '{}', row: {:?} column: {:?}, left: {}, right: {}, top: {}, bottom: {})",
            self.window.id, self.window.title, self.row, self.column, self.left, self.right, self.top, self.bottom
        ))
    }
}
