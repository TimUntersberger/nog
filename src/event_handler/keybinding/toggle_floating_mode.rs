use log::{debug,error};

use crate::{
    event::Event, system::NativeWindow, system::SystemResult,
    win_event_handler::win_event::WinEvent, win_event_handler::win_event_type::WinEventType,
    AppState,
};

pub fn handle(state: &mut AppState) -> SystemResult {
    let window = NativeWindow::get_foreground_window().expect("Failed to get foreground window");
    let config = state.config.clone();
    // The id of the grid that contains the window
    let maybe_grid_id = state
        .find_window(window.id)
        .and_then(|g| g.remove_by_window_id(window.id).map(|t| (g.id, t)))
        .map(|(id, mut window)| {
            debug!("Unmanaging window '{}' | {}", window.title, window.id);
            match window.cleanup() {
                Err(e) => error!("Error cleaning up window {} {:?}", window.id, e),
                _ => ()
            }

            id
        });

    if let Some(d) = maybe_grid_id.and_then(|id| state.find_grid(id)) {
        d.refresh_grid(&config)?;
    } else {
        state
            .event_channel
            .sender
            .clone()
            .send(Event::WinEvent(WinEvent {
                typ: WinEventType::Show(true),
                window,
            }))
            .expect("Failed to send WinEvent");
    }

    Ok(())
}
