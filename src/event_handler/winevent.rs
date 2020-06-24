mod destroy;
mod show;
mod focus_change;

use crate::WORK_MODE;
use winapi::shared::windef::HWND;
use crate::util;
use crate::win_event_handler::WinEvent;
use crate::win_event_handler::WinEventType;

use log::debug;

pub fn handle(ev: WinEvent) -> Result<(), Box<dyn std::error::Error>> {
    let title = match util::get_title_of_window(ev.hwnd as HWND) { // We only care about the windows that have a title
        Ok(title) => title,
        Err(_) => return Ok(())
    };

    debug!(
        "{:?}: '{}' | {}",
        ev.typ,
        title,
        ev.hwnd as i32
    );

    if *WORK_MODE.lock().unwrap() {
        match ev.typ {
            WinEventType::Destroy => destroy::handle(ev.hwnd as HWND)?,
            WinEventType::Show(ignore) => show::handle(ev.hwnd as HWND, ignore)?,
            WinEventType::FocusChange => focus_change::handle(ev.hwnd as HWND)?
        };
    }

    Ok(())
}
