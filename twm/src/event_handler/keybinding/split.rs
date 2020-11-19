use crate::{split_direction::SplitDirection, system::SystemResult, AppState};

pub fn handle(state: &mut AppState, direction: SplitDirection) -> SystemResult {
    let display = state.get_current_display_mut();
    if let Some(grid) = display.get_focused_grid_mut() {
        grid.set_focused_split_direction(direction);
    }
    Ok(())
}
