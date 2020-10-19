use crate::{popup, system::SystemResult, AppState};

pub fn handle(state: &mut AppState) -> SystemResult {
    if popup::is_visible() {
        popup::close()?;
        return Ok(());
    }

    let config = state.config.clone();
    let display = state.get_current_display_mut();
    if let Some(grid) = display.get_focused_grid_mut() {
        if let Some(id) = grid.get_focused_window().map(|w| {
            w.close();
            w.id
        }) {
            grid.remove_by_window_id(id);
            display.refresh_grid(&config)?;
        }
    }

    Ok(())
}
