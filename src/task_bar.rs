use crate::{display, util, CONFIG};
use lazy_static::lazy_static;
use log::{debug, info};
use std::collections::HashMap;
use std::sync::Mutex;
use winapi::shared::windef::HWND;
use winapi::shared::{
    minwindef::{BOOL, LPARAM},
    windef::RECT,
};
use winapi::um::winuser::EnumWindows;
use winapi::um::winuser::GetWindowRect;
use winapi::um::winuser::ShowWindow;
use winapi::um::winuser::SW_HIDE;
use winapi::um::winuser::{MonitorFromWindow, MONITOR_DEFAULTTONULL, SW_SHOW};

lazy_static! {
    // hmonitor, hwnd
    pub static ref WINDOWS: Mutex<HashMap<i32, i32>> = Mutex::new(HashMap::new());
    pub static ref HEIGHT: Mutex<i32> = Mutex::new(0);
}

unsafe extern "system" fn enum_windows_cb(hwnd: HWND, _: LPARAM) -> BOOL {
    let class_name = util::get_class_name_of_window(hwnd).expect("Failed to get class name");
    let is_task_bar = regex::Regex::new("^Shell_(Secondary)?TrayWnd$")
        .expect("Failed to build regex")
        .is_match(&class_name);

    if is_task_bar {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL);
        let mut rect = RECT::default();

        GetWindowRect(hwnd, &mut rect);

        *HEIGHT.lock().unwrap() = rect.bottom - rect.top;

        WINDOWS.lock().unwrap().insert(monitor as i32, hwnd as i32);

        debug!(
            "Initialized Taskbar(hwnd: {}, hmonitor: {})",
            hwnd as i32, monitor as i32
        );

        if !CONFIG.lock().unwrap().multi_monitor {
            return 0;
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
    foreach_taskbar(|hwnd| {
        info!("Showing taskbar {}", hwnd);
        unsafe {
            ShowWindow(hwnd as HWND, SW_SHOW);
        }
    });

}

fn foreach_taskbar(cb: fn(i32) -> ()) {
    let windows = WINDOWS.lock().unwrap();
    let mut monitors = windows.keys().collect::<Vec<&i32>>();

    monitors.sort_by(|x, y| {
        let display_x = display::get_display_by_hmonitor(**x);
        let display_y = display::get_display_by_hmonitor(**y);

        display_y.is_primary.cmp(&display_x.is_primary)
    });

    for hmonitor in monitors {
        let hwnd = *windows.get(hmonitor).expect("Failed to get hwnd of monitor");
        cb(hwnd);
    }
}

pub fn hide() {
    foreach_taskbar(|hwnd| {
        info!("Hiding taskbar {}", hwnd);
        unsafe {
            ShowWindow(hwnd as HWND, SW_HIDE);
        }
    });
}
