use crate::{direction::Direction, system::SystemResult, AppState};

pub fn handle(state: &mut AppState, direction: Direction) -> SystemResult {
    let config = state.config.clone();
    let display = state.get_current_display_mut();
    if let Some(grid) = display.get_focused_grid_mut() {
        if !config.ignore_fullscreen_actions || !grid.is_fullscreened() {
            grid.swap_focused(direction);
        }
    }
    display.refresh_grid(&config)?;
    Ok(())
}
