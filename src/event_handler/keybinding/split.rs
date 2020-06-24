use crate::tile_grid::SplitDirection;
use crate::GRIDS;
use crate::WORKSPACE_ID;

pub fn handle(direction: SplitDirection) -> Result<(), Box<dyn std::error::Error>> {
    let mut grids = GRIDS.lock().unwrap();
    grids
        .iter_mut()
        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
        .unwrap()
        .set_focused_split_direction(direction);

    Ok(())
}
