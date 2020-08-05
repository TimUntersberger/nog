use super::{RedrawAppBarReason, REDRAW_REASON, WINDOWS};
use winapi::{
    shared::windef::HWND,
    um::winuser::{SendMessageA, WM_PAINT},
};

pub fn redraw(reason: RedrawAppBarReason) {
    unsafe {
        *REDRAW_REASON.lock().unwrap() = reason;

        let hwnds: Vec<i32> = WINDOWS
            .lock()
            .unwrap()
            .iter()
            .map(|(_, hwnd)| *hwnd)
            .collect();

        for hwnd in hwnds {
            //TODO: handle error
            SendMessageA(hwnd as HWND, WM_PAINT, 0, 0);
        }
    }
}
