use crate::{direction::Direction, AppState};

pub fn handle(state: &AppState, direction: Direction) -> Result<(), Box<dyn std::error::Error>> {
    let display = state.get_current_display_mut();
    if let Some(grid) = display.get_focused_grid() {
        grid.swap(direction);
        display.refresh_grid(&state.config);
    }
    Ok(())
}
