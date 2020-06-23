use crate::WORKSPACE_ID;
use crate::GRIDS;
use crate::hot_key_manager::Direction;

pub fn handle(direction: Direction) -> Result<(), Box<dyn std::error::Error>> {
    let mut grids = GRIDS.lock().unwrap();
    let grid = grids
        .iter_mut()
        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
        .unwrap();

    grid.swap(direction)?;
    grid.draw_grid();

    Ok(())
}