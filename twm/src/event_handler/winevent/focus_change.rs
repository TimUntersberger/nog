use crate::{system::NativeWindow, system::SystemResult, AppState};

pub fn handle(state: &mut AppState, window: NativeWindow) -> SystemResult {
    if let Some(g) = state.find_grid_containing_window_mut(window.id) {
        g.focus_tile_by_window_id(window.id);
        state.workspace_id = g.id;
    }

    Ok(())
}
