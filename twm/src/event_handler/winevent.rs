use crate::{
    system::SystemResult,
    win_event_handler::{win_event::WinEvent, win_event_type::WinEventType},
    AppState,
};
use log::debug;

mod destroy;
mod focus_change;
mod show;

pub fn handle(state: &mut AppState, ev: WinEvent) -> SystemResult {
    let grids = state.get_grids_mut();
    let mut title: Option<String> = None;
    let mut grid_id: Option<i32> = None;

    for grid in grids.iter() {
        if let Some(window) = grid.get_window(ev.window.id) {
            title = Some(window.title.clone());
            grid_id = Some(grid.id);
            break;
        }
    }

    // window is not already managed and the event isn't `Show`
    if title.is_none() && ev.typ != WinEventType::Show(false) && ev.typ != WinEventType::Show(true)
    {
        return Ok(());
    }

    if title.is_none() {
        title = ev.window.get_title().ok();
    }

    if title.is_some() {
        debug!(
            "{:?}: '{}' | {}",
            ev.typ,
            ev.window.get_process_name(),
            ev.window.id
        );
    }

    match ev.typ {
        WinEventType::Destroy => destroy::handle(state, ev.window, grid_id)?,
        WinEventType::Show(ignore) => show::handle(state, ev.window, ignore)?,
        WinEventType::FocusChange => focus_change::handle(state, ev.window)?,
        WinEventType::Minimize => {
            if let Some(mut win) = state
                .find_grid_containing_window(ev.window.id)
                .and_then(|g| g.remove_by_window_id(ev.window.id))
            {
                win.cleanup()?;
                state.get_current_display().refresh_grid(&state.config)?;
            }
        },
        WinEventType::Hide
        | WinEventType::Unminimize => {}
    };

    Ok(())
}
