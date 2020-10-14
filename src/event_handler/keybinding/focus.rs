use crate::{direction::Direction, AppState, system::SystemResult};

pub fn handle(
    state: &mut AppState,
    direction: Direction,
) -> SystemResult {
    let config = state.config.clone();
    let display = state.get_current_display_mut();
    if let Some(grid) = display.get_focused_grid_mut() {
        grid.focus(direction)?;
        display.refresh_grid(&config);
    }
    Ok(())
}
