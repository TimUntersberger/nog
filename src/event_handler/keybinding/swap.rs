use crate::{direction::Direction, AppState};

pub fn handle(
    state: &mut AppState,
    direction: Direction,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = state.config.clone();
    let display = state.get_current_display_mut();
    if let Some(grid) = display.get_focused_grid_mut() {
        grid.swap(direction);
    }
    display.refresh_grid(&config);
    Ok(())
}
