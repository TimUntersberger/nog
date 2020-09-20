use crate::util;
use crate::{
    win_event_handler::{win_event::WinEvent, win_event_type::WinEventType},
    GRIDS,
};
use log::debug;

mod destroy;
mod focus_change;
mod show;

pub fn handle(ev: WinEvent) -> Result<(), Box<dyn std::error::Error>> {
    let grids = GRIDS.lock();
    let mut title: Option<String> = None;
    let mut grid_id: Option<i32> = None;

    for grid in grids.iter() {
        for tile in &grid.tiles {
            if tile.window.id == ev.window.id {
                title = Some(tile.window.title.clone());
                grid_id = Some(grid.id);
                break;
            }
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
        debug!("{:?}: '{}' | {}", ev.typ, title.unwrap(), ev.window.id);
    }

    drop(grids);

    match ev.typ {
        WinEventType::Destroy => destroy::handle(ev.window, grid_id)?,
        WinEventType::Show(ignore) => show::handle(ev.window, ignore)?,
        WinEventType::FocusChange => focus_change::handle(ev.window)?,
        WinEventType::Hide => {}
    };

    Ok(())
}
