use log::debug;

use crate::{
    event::Event, system::NativeWindow, win_event_handler::win_event::WinEvent,
    win_event_handler::win_event_type::WinEventType, AppState,
system::SystemResult};

pub fn handle(state: &mut AppState) -> SystemResult {
    let window = NativeWindow::get_foreground_window().expect("Failed to get foreground window");
    let config = state.config.clone();
    // The id of the grid that contains the window
    let maybe_grid_id = state
        .find_window(window.id)
        .and_then(|(g, _)| g.close_tile_by_window_id(window.id).map(|t| (g.id, t)))
        .map(|(id, mut t)| {
            debug!("Unmanaging window '{}' | {}", t.window.title, t.window.id);
            t.window.cleanup();

            id
        });

    if let Some((d, _)) = maybe_grid_id.and_then(|id| state.find_grid(id)) {
        d.refresh_grid(&config);
    } else {
        state
            .event_channel
            .sender
            .clone()
            .send(Event::WinEvent(WinEvent {
                typ: WinEventType::Show(true),
                window,
            })).expect("Failed to send WinEvent");
    }
    // if let Some((grid, _)) =  {
    //     let mut tile = grid.close_tile_by_window_id(window.id).unwrap();

    //     tile.window.cleanup();

    //     let display = state.get_current_display();

    //     debug!(
    //         "Unmanaging window '{}' | {}",
    //         tile.window.title, tile.window.id
    //     );

    //     grid.draw_grid(display, &state.config);
    //     display.refresh_grid(&state.config);
    // } else {
    //     state
    //         .event_channel
    //         .sender
    //         .clone()
    //         .send(Event::WinEvent(WinEvent {
    //             typ: WinEventType::Show(true),
    //             window,
    //         }))?;
    // }

    Ok(())
}
