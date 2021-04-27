use std::ptr;

use crate::{
    display::Display,
    keybindings::keybinding::Keybinding,
    system::DisplayId,
    system::Rectangle,
    system::SystemResult,
    system::WindowId,
    system::{api, SystemError},
    task_bar::Taskbar,
    util,
};
use log::{debug, error};
use regex::Regex;
use winapi::{
    shared::{minwindef::*, windef::*},
    um::{
        errhandlingapi::*, processthreadsapi::*, shellscalingapi::*, winbase::*, winnt::*,
        winreg::*, winuser::*,
    },
};

use super::{bool_to_result, nullable_to_result, Window};
use lazy_static::lazy_static;

lazy_static! {
    static ref TASKBAR_WINDOW_NAME_REGEX: Regex =
        Regex::new("^Shell_(Secondary)?TrayWnd$").expect("Failed to build taskbar name regex");
}

unsafe extern "system" fn monitor_cb(
    hmonitor: HMONITOR,
    _: HDC,
    _: LPRECT,
    l_param: LPARAM,
) -> BOOL {
    let displays = &mut *(l_param as *mut Vec<Display>);
    let display = Display::new(hmonitor.into());

    displays.push(display);

    1
}

pub fn get_last_error() -> String {
    let mut buffer = [0 as i8; 512];
    unsafe {
        FormatMessageA(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            ptr::null_mut(),
            GetLastError(),
            0,
            buffer.as_mut_ptr(),
            512,
            ptr::null_mut(),
        );
    }
    util::bytes_to_string(&buffer)
}

pub fn print_last_error() {
    error!("WINAPI ERROR: {}", get_last_error());
}

pub fn get_displays() -> Vec<Display> {
    let mut displays: Vec<Display> = Vec::new();
    unsafe {
        bool_to_result(EnumDisplayMonitors(
            ptr::null_mut(),
            ptr::null_mut(),
            Some(monitor_cb),
            &mut displays as *mut Vec<Display> as isize,
        ))
        .unwrap();
    }
    displays
}

pub fn get_display_dpi(id: DisplayId) -> u32 {
    let mut dpi_x: u32 = 0;
    let mut dpi_y: u32 = 0;

    unsafe {
        GetDpiForMonitor(id.into(), MDT_RAW_DPI, &mut dpi_x, &mut dpi_y);
    }

    dpi_x
}
unsafe extern "system" fn enum_windows_task_bars_cb(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let taskbars = &mut *(l_param as *mut Vec<Taskbar>);
    let mut window: Window = hwnd.into();
    let class_name = window.get_class_name().expect("Failed to get class name");
    let is_task_bar = TASKBAR_WINDOW_NAME_REGEX.is_match(&class_name);

    if is_task_bar {
        window
            .init(false, false)
            .expect("Failed to init taskbar window");
        taskbars.push(Taskbar::new(window));
    }

    1
}

pub fn get_display_rect(id: DisplayId) -> Rectangle {
    let mut monitor_info = MONITORINFO {
        cbSize: core::mem::size_of::<MONITORINFO>() as u32,
        ..MONITORINFO::default()
    };
    unsafe {
        GetMonitorInfoA(id.into(), &mut monitor_info);
    }
    monitor_info.rcMonitor.into()
}

pub fn get_taskbars() -> Vec<Taskbar> {
    let mut taskbars: Vec<Taskbar> = Vec::new();
    unsafe {
        EnumWindows(
            Some(enum_windows_task_bars_cb),
            &mut taskbars as *mut Vec<Taskbar> as isize,
        );
    }
    taskbars
}

pub fn add_launch_on_startup() {
    unsafe {
        let mut target_path = dirs::config_dir().unwrap();
        let source_path = std::env::current_exe().unwrap();

        target_path.push("nog");
        target_path.push("bin");
        target_path.push("nog.exe");

        if source_path != target_path {
            debug!("Exe doesn't exist yet");
            std::fs::copy(source_path, &target_path)
                .expect("Failed to copy executable to nog folder");
        }

        let mut key: HKEY = std::mem::zeroed();
        let mut key_name: Vec<u16> =
            util::to_widestring("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let mut value_name = util::to_widestring("nog");
        let app_path = util::to_widestring(target_path.to_str().unwrap());

        if RegCreateKeyExW(
            HKEY_CURRENT_USER,
            key_name.as_mut_ptr(),
            0,
            std::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            std::ptr::null_mut(),
            &mut key,
            std::ptr::null_mut(),
        ) == 0
        {
            RegSetValueExW(
                key,
                value_name.as_mut_ptr(),
                0,
                REG_SZ,
                app_path.as_ptr() as _,
                app_path.len() as u32 * 2,
            );
        };
    }
}

pub fn remove_launch_on_startup() {
    unsafe {
        let mut key_name: Vec<u16> =
            util::to_widestring("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let mut value_name = util::to_widestring("nog");

        RegDeleteKeyValueW(
            HKEY_CURRENT_USER,
            key_name.as_mut_ptr(),
            value_name.as_mut_ptr(),
        );
    }
}

pub fn register_keybinding(kb: &Keybinding) -> SystemResult {
    unsafe {
        let result = nullable_to_result(RegisterHotKey(
            std::ptr::null_mut(),
            kb.get_id(),
            kb.modifier.bits(),
            kb.key as u32,
        ));
        match result {
            Err(_) => Err(SystemError::RegisterKeybinding(format!("{:?}", kb))),
            _ => Ok(()),
        }
    }
}

pub fn unregister_keybinding(kb: &Keybinding) -> SystemResult {
    unsafe {
        let result = bool_to_result(UnregisterHotKey(std::ptr::null_mut(), kb.get_id()));
        match result {
            Err(_) => Err(SystemError::UnregisterKeybinding {
                key: format!("{:?}", kb),
                os_error: api::get_last_error(),
            }),
            _ => Ok(()),
        }
    }
}

pub fn unregister_keybinding_of_win(id: WindowId, kb: &Keybinding) {
    unsafe {
        UnregisterHotKey(id.into(), kb.get_id());
    }
}

pub fn get_current_window_msg() -> Option<MSG> {
    let mut msg: MSG = MSG::default();

    return unsafe {
        if PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);

            Some(msg)
        } else {
            None
        }
    };
}

pub fn launch_program(cmd: String) -> SystemResult {
    let mut si = STARTUPINFOA::default();
    let mut pi = PROCESS_INFORMATION::default();
    let mut cmd_bytes: Vec<u8> = cmd.bytes().chain(std::iter::once(0)).collect();

    unsafe {
        let x = CreateProcessA(
            std::ptr::null_mut(),
            cmd_bytes.as_mut_ptr() as *mut i8,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut si,
            &mut pi,
        );

        if x != 1 {
            Err(SystemError::LaunchProgram(cmd))
        } else {
            Ok(())
        }
    }
}
