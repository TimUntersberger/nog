use crate::{direction::Direction, AppState};
use log::info;

pub fn handle(
    state: &mut AppState,
    direction: Direction,
    amount: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = state.config.clone();
    let display = state.get_current_display_mut();
    if let Some(grid) = display.get_focused_grid_mut() {
        if let Some(tile) = grid.get_focused_tile() {
            match direction {
                Direction::Left | Direction::Right => tile
                    .column
                    .map(|v| grid.resize_column(v, direction, amount)),
                _ => tile.row.map(|v| grid.resize_row(v, direction, amount)),
            };

            info!("Resizing in the direction {:?} by {}", direction, amount);

            display.refresh_grid(&config);
        }
    }
    Ok(())
}
