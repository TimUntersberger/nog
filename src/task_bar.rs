use crate::{util, CONFIG};
use lazy_static::lazy_static;
use log::debug;
use std::collections::HashMap;
use winapi::um::winuser::EnumWindows;
use std::sync::Mutex;
use winapi::shared::windef::HWND;
use winapi::shared::{minwindef::{BOOL, LPARAM}, windef::RECT};
use winapi::um::winuser::GetWindowRect;
use winapi::um::winuser::ShowWindow;
use winapi::um::winuser::SW_HIDE;
use winapi::um::winuser::{MonitorFromWindow, SW_SHOW, MONITOR_DEFAULTTONULL};

lazy_static! {
    // hmonitor, hwnd
    pub static ref WINDOWS: Mutex<HashMap<i32, i32>> = Mutex::new(HashMap::new());
    pub static ref HEIGHT: Mutex<i32> = Mutex::new(0);
}

unsafe extern "system" fn enum_windows_cb(hwnd: HWND, _: LPARAM) -> BOOL {
    let class_name = util::get_class_name_of_window(hwnd).expect("Failed to get class name");
    let is_task_bar = regex::Regex::new("^Shell_(Secondary)?TrayWnd$").expect("Failed to build regex").is_match(&class_name);

    if is_task_bar {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL);
        let mut rect = RECT::default();

        GetWindowRect(hwnd, &mut rect);

        *HEIGHT.lock().unwrap() = rect.bottom - rect.top;

        WINDOWS
            .lock()
            .unwrap()
            .insert(monitor as i32, hwnd as i32);

        debug!(
            "Initialized Taskbar(hwnd: {}, hmonitor: {})",
            hwnd as i32, monitor as i32
        );

        if !CONFIG.lock().unwrap().multi_monitor {
            return 0
        }
    }

    1
}

pub fn init() {
    unsafe {
        EnumWindows(Some(enum_windows_cb), 0);
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
