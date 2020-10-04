use crate::{system::NativeWindow, AppState};

pub fn handle(
    state: &AppState,
    window: NativeWindow,
    grid_id: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let display = state.get_current_display_mut();
    if let Some(grid) = display.get_focused_grid() {
        if grid.close_tile_by_window_id(window.id).is_some() {
            display.refresh_grid(&state.config);
        }
    }
    Ok(())
}
