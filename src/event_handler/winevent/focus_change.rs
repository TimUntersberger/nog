use crate::{system::NativeWindow, with_current_grid};

pub fn handle(window: NativeWindow) -> Result<(), Box<dyn std::error::Error>> {
    with_current_grid(|grid| {
        if let Some(id) = grid.focused_window_id {
            if window.id == id {
                return Ok(());
            }

            if grid.get_tile_by_id(window.id).is_some() {
                grid.focus_stack.clear();
                grid.focused_window_id = Some(window.id);
            }
        }

        Ok(())
    })
}
