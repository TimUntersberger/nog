use super::{draw_datetime, draw_workspaces, WINDOWS};
use winapi::{
    shared::windef::HWND,
    um::winuser::{ShowWindow, SW_HIDE, SW_SHOW},
};

#[allow(dead_code)]
pub fn hide() {
    unsafe {
        let hwnds: Vec<i32> = WINDOWS
            .lock()
            .unwrap()
            .iter()
            .map(|(_, hwnd)| *hwnd)
            .collect();
        for hwnd in hwnds {
            ShowWindow(hwnd as HWND, SW_HIDE);
        }
    }
}

pub fn show() {
    unsafe {
        let hwnds: Vec<i32> = WINDOWS
            .lock()
            .unwrap()
            .iter()
            .map(|(_, hwnd)| *hwnd)
            .collect();
        for hwnd in hwnds {
            ShowWindow(hwnd as HWND, SW_SHOW);
            draw_workspaces::draw(hwnd as HWND);
            draw_datetime::draw(hwnd as HWND).expect("Failed to draw datetime");
        }
    }
}
