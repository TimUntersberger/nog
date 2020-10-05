use crate::{system::NativeWindow, AppState};

pub fn handle(
    state: &mut AppState,
    window: NativeWindow,
    grid_id: Option<i32>, // TODO: maybe remove this? IDK
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(_) = state
        .find_window(window.id)
        .map(|(g, _)| g.close_tile_by_window_id(window.id))
    {
        state.get_current_display().refresh_grid(&state.config);
    }
    Ok(())
}
