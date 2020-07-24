use crate::util;
use crate::win_event_handler::WinEvent;
use crate::win_event_handler::WinEventType;
use crate::GRIDS;
use log::debug;
use winapi::shared::windef::HWND;

mod destroy;
mod focus_change;
mod show;

pub fn handle(ev: WinEvent) -> Result<(), Box<dyn std::error::Error>> {
    let grids = GRIDS.lock().unwrap();
    let mut title: Option<String> = None;
    let mut grid_id: Option<i32> = None;

    for grid in grids.iter() {
        for tile in &grid.tiles {
            if tile.window.id == ev.hwnd {
                title = Some(tile.window.title.clone());
                grid_id = Some(grid.id);
                break;
            }
        }
    }

    if title.is_none() && ev.typ != WinEventType::Show(false) && ev.typ != WinEventType::Show(true)
    {
        return Ok(());
    }

    if title.is_none() {
        title = util::get_title_of_window(ev.hwnd as HWND).ok();
    }

    if title.is_some() {
        debug!("{:?}: '{}' | {}", ev.typ, title.unwrap(), ev.hwnd as i32);
    }

    drop(grids);

    match ev.typ {
        WinEventType::Destroy => destroy::handle(ev.hwnd as HWND, grid_id)?,
        WinEventType::Show(ignore) => show::handle(ev.hwnd as HWND, ignore)?,
        WinEventType::FocusChange => focus_change::handle(ev.hwnd as HWND)?,
        WinEventType::Hide => {}
    };

    Ok(())
}
