use crate::GRIDS;
use crate::WORKSPACE_ID;
use winapi::shared::windef::HWND;

pub fn handle(hwnd: HWND) -> Result<(), Box<dyn std::error::Error>> {
    let mut grids = GRIDS.lock().unwrap();
    let mut grid = grids
        .iter_mut()
        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
        .unwrap();

    if let Some(id) = grid.focused_window_id {
        if hwnd == id as HWND {
            return Ok(());
        }

        if grid.get_tile_by_id(hwnd as i32).is_some() {
            grid.focus_stack.clear();
            grid.focused_window_id = Some(hwnd as i32);
        }
    }

    Ok(())
}
