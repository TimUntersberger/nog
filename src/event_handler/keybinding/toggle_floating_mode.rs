use log::debug;

use crate::{
    event::Event, system::NativeWindow, win_event_handler::win_event::WinEvent,
    win_event_handler::win_event_type::WinEventType, AppState,
};

pub fn handle(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let window = NativeWindow::get_foreground_window()?;

    if let Some((grid, tile)) = state.find_window(window.id) {
        let display = state.get_current_display_mut();
        if display.focused_grid_id == Some(grid.id) {
            debug!(
                "Reseting window '{}' | {}",
                tile.window.title, tile.window.id
            );

            tile.window.cleanup();

            debug!(
                "Unmanaging window '{}' | {}",
                tile.window.title, tile.window.id
            );

            grid.close_tile_by_window_id(tile.window.id);
            display.refresh_grid(&state.config);
        }
    } else {
        state
            .event_channel
            .sender
            .clone()
            .send(Event::WinEvent(WinEvent {
                typ: WinEventType::Show(true),
                window,
            }))?;
    }

    Ok(())
}
