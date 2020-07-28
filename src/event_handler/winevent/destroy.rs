use crate::GRIDS;
use crate::WORKSPACE_ID;
use winapi::shared::windef::HWND;

pub fn handle(hwnd: HWND, grid_id: Option<i32>) -> Result<(), Box<dyn std::error::Error>> {
    let mut grids = GRIDS.lock().unwrap();
    let grid_id = grid_id.unwrap_or(*WORKSPACE_ID.lock().unwrap());
    let grid = grids
        .iter_mut()
        .find(|g| g.id == grid_id)
        .unwrap();

    if grid.close_tile_by_window_id(hwnd as i32).is_some() {
        grid.draw_grid();
    }

    Ok(())
}
