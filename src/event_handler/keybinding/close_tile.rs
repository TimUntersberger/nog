use crate::{popup, AppState};

pub fn handle(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    if popup::is_visible() {
        popup::close();
        return Ok(());
    }

    let display = state.get_current_display_mut();
    if let Some(grid) = display.get_focused_grid() {
        if let Some(tile) = grid.get_focused_tile() {
            tile.window.close()?;
            grid.close_tile_by_window_id(tile.window.id);
            display.refresh_grid(&state.config);
        }
    }

    Ok(())
}
