use crate::{direction::Direction, with_current_grid};
use log::info;

pub fn handle(direction: Direction, amount: i32) -> Result<(), Box<dyn std::error::Error>> {
    with_current_grid(|grid| {
        let tile = grid
            .get_focused_tile()
            .ok_or("Failed to get focused tile")?;
        let column = tile.column;
        let row = tile.row;

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
    })
}
