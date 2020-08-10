use super::{get_windows, redraw::redraw};
use winapi::{
    shared::windef::HWND,
    um::winuser::{ShowWindow, SW_HIDE, SW_SHOW},
};

#[allow(dead_code)]
pub fn hide() {
    unsafe {
        for hwnd in get_windows() {
            ShowWindow(hwnd as HWND, SW_HIDE);
        }
    }
}

pub fn show() {
    unsafe {
        for hwnd in get_windows() {
            ShowWindow(hwnd as HWND, SW_SHOW);
        }

        redraw();
    }
}
