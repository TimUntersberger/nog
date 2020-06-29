use crate::DISPLAYS;
use lazy_static::lazy_static;
use log::debug;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::Mutex;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::winuser::FindWindowA;
use winapi::um::winuser::GetWindowRect;
use winapi::um::winuser::ShowWindow;
use winapi::um::winuser::SW_HIDE;
use winapi::um::winuser::SW_SHOW;

lazy_static! {
    pub static ref WINDOWS: Mutex<HashMap<i32, i32>> = Mutex::new(HashMap::new());
    pub static ref HEIGHT: Mutex<i32> = Mutex::new(0);
}

pub fn init() {
    for (i, display) in DISPLAYS.lock().unwrap().iter().enumerate() {
        let mut rect = RECT::default();
        let window_name = if i == 0 {
            CString::new("Shell_TrayWnd").unwrap()
        } else {
            CString::new("Shell_SecondaryTrayWnd").unwrap()
        };

        let window_handle = unsafe { FindWindowA(window_name.as_ptr(), std::ptr::null()) };
        unsafe {
            GetWindowRect(window_handle, &mut rect);
        }

        if i == 0 {
            *HEIGHT.lock().unwrap() = rect.bottom - rect.top;
        }

        WINDOWS
            .lock()
            .unwrap()
            .insert(display.hmonitor as i32, window_handle as i32);

        debug!(
            "Initialized Taskbar(hwnd: {}, hmonitor: {})",
            window_handle as i32, display.hmonitor as i32
        );
    }
}

pub fn show() {
    debug!("Showing taskbar");
    let hwnds: Vec<i32> = WINDOWS
        .lock()
        .unwrap()
        .iter()
        .map(|(_, hwnd)| *hwnd)
        .collect();

    for hwnd in hwnds {
        unsafe {
            ShowWindow(hwnd as HWND, SW_SHOW);
        }
    }
}

pub fn hide() {
    debug!("Hiding taskbar");
    let hwnds: Vec<i32> = WINDOWS
        .lock()
        .unwrap()
        .iter()
        .map(|(_, hwnd)| *hwnd)
        .collect();

    for hwnd in hwnds {
        unsafe {
            ShowWindow(hwnd as HWND, SW_HIDE);
        }
    }
}
