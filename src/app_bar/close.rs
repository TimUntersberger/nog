use super::{FONT, WINDOWS};
use log::{debug, info};
use std::ffi::CString;
use winapi::shared::windef::HWND;
use winapi::um::winuser::{SendMessageA, UnregisterClassA, WM_CLOSE};

pub fn close() {
    unsafe {
        info!("Closing appbar");

        let windows: Vec<(i32, i32)> = WINDOWS
            .lock()
            .unwrap()
            .iter()
            .map(|(hmonitor, hwnd)| (*hmonitor, *hwnd))
            .collect();

        for (hmonitor, hwnd) in windows {
            SendMessageA(hwnd as HWND, WM_CLOSE, 0, 0);
            WINDOWS.lock().unwrap().remove(&hmonitor);
        }
        let name = CString::new("nog_bar").expect("Failed to transform string to cstring");

        debug!("Unregistering window class");

        UnregisterClassA(
            name.as_ptr(),
            winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut()),
        );

        *FONT.lock().unwrap() = 0;
    }
}
