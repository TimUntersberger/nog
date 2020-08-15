use crate::{split_direction::SplitDirection, with_current_grid};

pub fn handle(direction: SplitDirection) -> Result<(), Box<dyn std::error::Error>> {
    with_current_grid(|grid| {
        grid.set_focused_split_direction(direction);
    });
    Ok(())
}
