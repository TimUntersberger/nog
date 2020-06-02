use crate::window::Window;
use crate::tile_grid::SplitDirection;

#[derive(Clone)]
pub struct Tile {
    pub column: Option<i32>,
    pub row: Option<i32>,
    pub split_direction: SplitDirection,
    pub window: Window
}

impl Tile {
    pub fn new(window: Window) -> Self {
        Self {
            column: None,
            row: None,
            split_direction: SplitDirection::Vertical,
            window: window
        }
    }
}