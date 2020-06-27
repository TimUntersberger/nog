use crate::tile_grid::SplitDirection;
use crate::window::Window;

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
