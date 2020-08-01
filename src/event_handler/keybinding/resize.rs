use crate::hot_key_manager::Direction;
use crate::GRIDS;
use crate::WORKSPACE_ID;
use log::{info};

pub fn handle(direction: Direction, amount: i32) -> Result<(), Box<dyn std::error::Error>> {
    let mut grids = GRIDS.lock().unwrap();
    let grid = grids
        .iter_mut()
        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
        .unwrap();

    let tile = grid.get_focused_tile_mut().expect("Failed to get focused tile");

    match direction {
        Direction::Left => if tile.column != None { tile.left += amount },
        Direction::Right => if tile.column != None { tile.right += amount },
        Direction::Up => if tile.row != None { tile.top += amount },
        Direction::Down => if tile.row != None { tile.bottom += amount }
    }

    info!("Resizing tile in the direction {:?} by {}", direction, amount);

    grid.draw_grid();

    Ok(())
}
