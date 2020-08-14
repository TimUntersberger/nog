use super::get_windows;
use winapi::{
    shared::windef::HWND,
    um::winuser::{SendMessageA, WM_PAINT},
};

pub fn redraw() {
    unsafe {
        for window in get_windows() {
            window.redraw();
        }
    }
}
