use winapi::um::winuser::FindWindowA;
use winapi::um::winuser::GetWindowRect;
use winapi::um::winuser::SW_HIDE;
use winapi::um::winuser::SW_SHOW;
use winapi::um::winuser::ShowWindow;
use winapi::shared::windef::RECT;
use winapi::shared::windef::HWND;

pub static mut X: i32 = 0;
pub static mut Y: i32 = 0;
pub static mut WINDOW: i32 = 0;
pub static mut HEIGHT: i32 = 0;
pub static mut WIDTH: i32 = 0;

pub unsafe fn set_hwnd(hwnd: HWND) {
    WINDOW = hwnd as i32;
}

pub unsafe fn init(){
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0
    };
    GetWindowRect(WINDOW as HWND, &mut rect);
    X = rect.left;
    Y = rect.top;
    HEIGHT = rect.bottom - rect.top;
    WIDTH = rect.right - rect.left;
}

pub unsafe fn show(){
    ShowWindow(WINDOW as HWND, SW_SHOW);
}

pub unsafe fn hide(){
    println!("{}", WINDOW);
    ShowWindow(WINDOW as HWND, SW_HIDE);
}