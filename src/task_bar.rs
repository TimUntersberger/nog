use lazy_static::lazy_static;
use log::debug;
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
    pub static ref X: Mutex<i32> = Mutex::new(0);
    pub static ref Y: Mutex<i32> = Mutex::new(0);
    pub static ref WINDOW: Mutex<i32> = Mutex::new(0);
    pub static ref HEIGHT: Mutex<i32> = Mutex::new(0);
    pub static ref WIDTH: Mutex<i32> = Mutex::new(0);
}

pub fn init() {
    let mut rect = RECT::default();
    let window_name = CString::new("Shell_TrayWnd").unwrap();

    let mut gwindow = WINDOW.lock().unwrap();
    let mut gx = X.lock().unwrap();
    let mut gy = Y.lock().unwrap();
    let mut gwidth = WIDTH.lock().unwrap();
    let mut gheight = HEIGHT.lock().unwrap();

    unsafe {
        *gwindow = FindWindowA(window_name.as_ptr(), std::ptr::null()) as i32;
        GetWindowRect(*gwindow as HWND, &mut rect);

        *gx = rect.left;
        *gy = rect.top;
        *gheight = rect.bottom - rect.top;
        *gwidth = rect.right - rect.left;

        debug!(
            "Initialized Taskbar(x: {}, y: {}, width: {}, height: {})",
            *gx, *gy, *gwidth, *gheight
        );
    }
}

pub fn show() {
    debug!("Showing taskbar");
    unsafe {
        ShowWindow(*WINDOW.lock().unwrap() as HWND, SW_SHOW);
    }
}

pub fn hide() {
    debug!("Hiding taskbar");
    unsafe {
        ShowWindow(*WINDOW.lock().unwrap() as HWND, SW_HIDE);
    }
}
