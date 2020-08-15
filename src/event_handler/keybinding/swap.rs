use crate::{direction::Direction, with_current_grid};

pub fn handle(direction: Direction) -> Result<(), Box<dyn std::error::Error>> {
    with_current_grid(|grid| {
        grid.swap(direction)?;
        grid.draw_grid();

        Ok(())
    })
}
