use crate::{util, CONFIG, DISPLAYS};
use log::{debug, info};
use std::mem::size_of;
use winapi::shared::windef::HWND;
use winapi::shared::{
    minwindef::{BOOL, LPARAM},
    windef::HMONITOR,
    windef::RECT,
};
use winapi::um::winuser::{
    EnumWindows, GetMonitorInfoA, GetWindowRect, IsWindowVisible, MonitorFromWindow, ShowWindow,
    MONITORINFO, MONITOR_DEFAULTTONULL, SW_HIDE, SW_SHOW,
};

#[derive(Debug, Clone, Copy)]
pub enum TaskBarPosition {
    Top,
    Bottom,
    Left,
    Right,
    Hidden,
}

impl Default for TaskBarPosition {
    fn default() -> Self {
        TaskBarPosition::Bottom
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct TaskBar {
    pub hwnd: i32,
    pub height: i32,
    pub width: i32,
    pub position: TaskBarPosition,
}

pub fn show_taskbars() {
    foreach_taskbar(|hwnd| {
        info!("Showing taskbar {}", hwnd);
        unsafe {
            ShowWindow(hwnd as HWND, SW_SHOW);
        }
    });

    update_task_bars();
}
pub fn hide_taskbars() {
    foreach_taskbar(|hwnd| {
        info!("Hiding taskbar {}", hwnd);
        unsafe {
            ShowWindow(hwnd as HWND, SW_HIDE);
        }
    });

    update_task_bars();
}
fn foreach_taskbar(cb: fn(i32) -> ()) {
    let mut displays = DISPLAYS.lock();
    displays.sort_by(|x, y| y.is_primary.cmp(&x.is_primary));

    let displays = displays.iter().filter(|x| x.task_bar.is_some());

    for display in displays {
        cb(display.task_bar.unwrap().hwnd);
    }
}

pub fn update_task_bars() {
    unsafe {
        EnumWindows(Some(enum_windows_cb), 0);
    }
}

unsafe extern "system" fn enum_windows_cb(hwnd: HWND, _: LPARAM) -> BOOL {
    let class_name = util::get_class_name_of_window(hwnd).expect("Failed to get class name");
    let is_task_bar = regex::Regex::new("^Shell_(Secondary)?TrayWnd$")
        .expect("Failed to build regex")
        .is_match(&class_name);

    if is_task_bar {
        let mut rect = RECT::default();

        GetWindowRect(hwnd, &mut rect);

        let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL) as i32;
        let task_bar = TaskBar {
            hwnd: hwnd as i32,
            height: rect.bottom - rect.top,
            width: rect.right - rect.left,
            position: get_taskbar_position(rect, hwnd, hmonitor),
        };

        debug!("Initialized {:?})", task_bar);

        let is_display_primary = match DISPLAYS
            .lock()
            
            .iter_mut()
            .find(|d| d.hmonitor == hmonitor)
        {
            Some(d) => {
                d.task_bar = Some(task_bar);
                d.is_primary
            }
            _ => false,
        };

        if !CONFIG.lock().multi_monitor && is_display_primary {
            return 0;
        }
    }

    1
}

fn get_taskbar_position(rect: RECT, hwnd: HWND, hmonitor: i32) -> TaskBarPosition {
    let mut monitor_info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..MONITORINFO::default()
    };
    unsafe {
        GetMonitorInfoA(hmonitor as HMONITOR, &mut monitor_info);
    }
    let RECT {
        left,
        right,
        top,
        bottom,
    } = monitor_info.rcMonitor;
    let is_window_visible = unsafe { IsWindowVisible(hwnd) == 1 };

    if !is_window_visible {
        TaskBarPosition::Hidden
    } else if rect.left == left && rect.top == top && rect.bottom == bottom {
        TaskBarPosition::Left
    } else if rect.right == right && rect.top == top && rect.bottom == bottom {
        TaskBarPosition::Right
    } else if rect.left == left && rect.top == top && rect.right == right {
        TaskBarPosition::Top
    } else {
        TaskBarPosition::Bottom
    }
}
