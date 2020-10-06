use crate::{split_direction::SplitDirection, AppState};

pub fn handle(
    state: &mut AppState,
    direction: SplitDirection,
) -> Result<(), Box<dyn std::error::Error>> {
    let display = state.get_current_display_mut();
    if let Some(grid) = display.get_focused_grid_mut() {
        grid.set_focused_split_direction(direction);
    }
    Ok(())
}
