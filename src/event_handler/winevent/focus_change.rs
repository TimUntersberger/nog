use crate::with_current_grid;
use winapi::shared::windef::HWND;

pub fn handle(hwnd: HWND) -> Result<(), Box<dyn std::error::Error>> {
    with_current_grid(|grid| {
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
    })
}
