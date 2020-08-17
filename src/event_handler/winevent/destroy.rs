use crate::with_grid_by_id;
use crate::WORKSPACE_ID;
use winapi::shared::windef::HWND;

pub fn handle(hwnd: HWND, grid_id: Option<i32>) -> Result<(), Box<dyn std::error::Error>> {
    with_grid_by_id(grid_id.unwrap_or(*WORKSPACE_ID.lock().unwrap()), |grid| {
        if grid.close_tile_by_window_id(hwnd as i32).is_some() {
            grid.draw_grid();
        }

        Ok(())
    })
}
