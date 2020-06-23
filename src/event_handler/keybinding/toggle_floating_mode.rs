use crate::window::Window;
use crate::GRIDS;
use crate::CHANNEL;
use crate::event::Event;
use crate::win_event_handler::WinEvent;
use crate::win_event_handler::WinEventType;
use crate::WORKSPACE_ID;

use log::debug;

pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let window_handle = Window::get_foreground_window()?;

    let mut grids = GRIDS.lock().unwrap();
    let gid = *WORKSPACE_ID.lock().unwrap();

    // May have a grid that has the window as tile
    let maybe_grid = grids
        .iter_mut()
        .filter(|g| g.visible) // only care about the workspaces that are used
        .map(|g| {
            (g.get_focused_tile().clone(), g.id)
        }) // (maybe_focused_tile, grid_id)
        .filter(|t| t.0.is_some()) // check whether it is safe to unwrap
        .map(|t| (t.1, t.0.unwrap())) // unwrap focused_tile -> (grid_id, focused_tile)
        .find(|t| t.1.window.id == window_handle as i32); // find me the tuple that has the window

    if let Some(tuple) = maybe_grid {
        let grid_id = tuple.0;
        let focused_tile = tuple.1;
        let focused_tile_id = focused_tile.window.id;

        if grid_id == gid { // only continue if the tuple is currently visible
            debug!(
                "Reseting window '{}' | {}",
                focused_tile.window.title, focused_tile.window.id
            );

            focused_tile.window.reset_style()?;
            focused_tile.window.reset_pos()?;

            debug!(
                "Unmanaging window '{}' | {}",
                focused_tile.window.title, focused_tile.window.id
            );

            let grid = grids
                .iter_mut()
                .find(|g| g.id == gid)
                .unwrap();

            grid.close_tile_by_window_id(focused_tile_id);
            grid.draw_grid();
        }
    } else {
        CHANNEL.sender.clone().send(Event::WinEvent(WinEvent {
            typ: WinEventType::Show(true),
            hwnd: window_handle as i32
        }))?;
    }

    Ok(())
}
