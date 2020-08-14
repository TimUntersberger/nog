use crate::GRIDS;
use crate::WORKSPACE_ID;

pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let mut grids = GRIDS.lock().unwrap();
    let grid = grids
        .iter_mut()
        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
        .unwrap();

    if let Some(tile) = grid.get_focused_tile() {
        tile.window.close();
        let id = tile.window.id; //need this variable because of borrow checker
        grid.close_tile_by_window_id(id);
        grid.draw_grid();
    }

    Ok(())
}
