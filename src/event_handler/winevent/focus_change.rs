use crate::{system::NativeWindow, AppState};

pub fn handle(state: &AppState, window: NativeWindow) -> Result<(), Box<dyn std::error::Error>> {
    let display = state.get_current_display_mut();
    if let Some(grid) = display.get_focused_grid() {
        if let Some(id) = grid.focused_window_id {
            if window.id != id && grid.get_tile_by_id(window.id).is_some() {
                grid.focus_stack.clear();
                grid.focused_window_id = Some(window.id);
            }
        }
    }

    Ok(())
}
