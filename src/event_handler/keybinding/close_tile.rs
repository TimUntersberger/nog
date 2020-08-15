use crate::{popup, with_current_grid};

pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    if popup::is_visible() {
        popup::close();
        return Ok(())
    }

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
