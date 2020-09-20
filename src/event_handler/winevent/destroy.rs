use crate::WORKSPACE_ID;
use crate::{system::NativeWindow, with_grid_by_id};

pub fn handle(
    window: NativeWindow,
    grid_id: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    with_grid_by_id(grid_id.unwrap_or(*WORKSPACE_ID.lock()), |grid| {
        if grid.close_tile_by_window_id(window.id).is_some() {
            grid.draw_grid();
        }

        Ok(())
    })
}
