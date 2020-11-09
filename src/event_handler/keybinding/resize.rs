use crate::{direction::Direction, system::SystemResult, AppState};
use log::info;

pub fn handle(state: &mut AppState, direction: Direction, amount: i32) -> SystemResult {
    let config = state.config.clone();
    let display = state.get_current_display_mut();
    if let Some(grid) = display.get_focused_grid_mut() {
        if !config.ignore_fullscreen_actions || !grid.is_fullscreened() {
            grid.trade_size_with_neighbor(grid.focused_id, direction, amount);
            info!("Resizing in the direction {:?} by {}", direction, amount);
            display.refresh_grid(&config)?;
        }
    }
    Ok(())
}
