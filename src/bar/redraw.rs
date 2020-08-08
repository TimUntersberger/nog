use super::{get_windows, WINDOWS};
use winapi::{
    shared::windef::HWND,
    um::winuser::{SendMessageA, WM_PAINT},
};

pub fn redraw() {
    unsafe {
        for hwnd in get_windows() {
            //TODO: handle error
            SendMessageA(hwnd as HWND, WM_PAINT, 0, 0);
        }
    }
}
