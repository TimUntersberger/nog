use crate::with_current_grid;

pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    with_current_grid(|grid| {
        if let Some(tile) = grid.get_focused_tile() {
            tile.window.close();
            let id = tile.window.id; //need this variable because of borrow checker
            grid.close_tile_by_window_id(id);
            grid.draw_grid();
        }
    });

    Ok(())
}
