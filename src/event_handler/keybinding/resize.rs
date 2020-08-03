use crate::GRIDS;
use crate::{direction::Direction, WORKSPACE_ID};
use log::info;

pub fn handle(direction: Direction, amount: i32) -> Result<(), Box<dyn std::error::Error>> {
    let mut grids = GRIDS.lock().unwrap();
    let grid = grids
        .iter_mut()
        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
        .unwrap();

    let tile = grid.get_focused_tile().ok_or("Failed to get focused tile")?;
    let column = tile.column.clone();
    let row = tile.row.clone();
    drop(tile);

    if direction == Direction::Left || direction == Direction::Right {
        if let Some(value) = column {
            grid.resize_column(value, direction, amount);
        }
    } else {
        if let Some(value) = row {
            grid.resize_row(value, direction, amount);
        }
    }

    info!("Resizing in the direction {:?} by {}", direction, amount);

    grid.draw_grid();

    Ok(())
}
