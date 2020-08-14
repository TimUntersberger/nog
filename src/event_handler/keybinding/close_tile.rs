use crate::GRIDS;
use crate::{popup, WORKSPACE_ID};

pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    if popup::is_visible() {
        popup::close();
        return Ok(())
    }

    let mut grids = GRIDS.lock().unwrap();
    let grid = grids
        .iter_mut()
        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
        .unwrap();

    if let Some(tile) = grid.get_focused_tile() {
        tile.window.send_close();
        let id = tile.window.id; //need this variable because of borrow checker
        grid.close_tile_by_window_id(id);
        grid.draw_grid();
    }

    Ok(())
}
