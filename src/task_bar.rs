use winapi::shared::minwindef::BOOL;
use winapi::um::winuser::GetWindowRect;
use winapi::um::winuser::SW_HIDE;
use winapi::um::winuser::SW_SHOW;
use winapi::um::winuser::ShowWindow;
use winapi::um::shellapi::SHAppBarMessage;
use winapi::um::shellapi::APPBARDATA;
use winapi::um::shellapi::ABM_GETTASKBARPOS;
use winapi::shared::windef::RECT;
use winapi::shared::windef::HWND;

use crate::util;

pub static mut X: i32 = 0;
pub static mut Y: i32 = 0;
pub static mut WINDOW: i32 = 0;
pub static mut HEIGHT: i32 = 0;
pub static mut WIDTH: i32 = 0;

pub fn set_hwnd(hwnd: HWND) {
    unsafe {
        WINDOW = hwnd as i32;
    }
}

pub fn init() -> util::WinApiResult<BOOL> {
    let mut rect = RECT::default(); 

    unsafe {
        let mut data = APPBARDATA::default();
        SHAppBarMessage(ABM_GETTASKBARPOS, &mut data);
        println!("{}", winapi::um::winuser::FindWindowA("Shell_TrayWnd".as_ptr() as *const i8, std::ptr::null()) as i32);
        //WINDOW = winapi::um::winuser::FindWindowA("Shell_TrayWnd".as_ptr() as *const i8, std::ptr::null()) as i32;
        // util::winapi_err_to_result(GetWindowRect(WINDOW as HWND, &mut rect))?;
        let rect = data.rc;
        X = rect.left;
        Y = rect.top;
        HEIGHT = rect.bottom - rect.top;
        WIDTH = rect.right - rect.left;
    }

    Ok(1)
}

pub fn show() -> util::WinApiResult<BOOL> {
    unsafe {
        return util::winapi_err_to_result(ShowWindow(WINDOW as HWND, SW_SHOW));
    }
}

pub fn hide() -> util::WinApiResult<BOOL> {
    unsafe {
        return util::winapi_err_to_result(ShowWindow(WINDOW as HWND, SW_HIDE));
    }
}