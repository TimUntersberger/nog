use crate::{
    display::get_primary_display, event::Event, message_loop, task_bar::HEIGHT, util,
    window::Window, CHANNEL, CONFIG, DISPLAYS,
};
use log::{debug, error, info};
use std::{
    collections::HashMap,
    ffi::CString,
    sync::{Mutex, MutexGuard},
    thread,
};
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::wingdi::SetBkColor;
use winapi::um::wingdi::SetTextColor;
use winapi::um::wingdi::TRANSPARENT;
use winapi::um::winuser::{
    BeginPaint, DefWindowProcA, EndPaint, GetCursorPos, GetDC, LoadCursorA, RegisterClassA,
    ReleaseDC, SetCursor, ShowWindow, IDC_ARROW, PAINTSTRUCT, SW_SHOW, WM_PAINT, WM_SETCURSOR,
};
use winapi::{
    shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM},
    um::winuser::{DrawTextW, DT_CENTER, DT_SINGLELINE, DT_VCENTER, WNDCLASSA},
};

use lazy_static::lazy_static;

lazy_static! {
    static ref POPUP: Mutex<Option<Popup>> = Mutex::new(None);
}

#[derive(Clone)]
pub struct Popup {
    pub window: Window,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub text: Vec<String>,
}

impl Popup {
    pub fn new(name: String, width: i32, height: i32) -> Self {
        Self {
            window: Window::default(),
            name,
            width,
            height,
            text: Vec::new(),
        }
    }

    pub fn with_text(&mut self, text: &[String]) -> &mut Self {
        self.text = text.to_vec();
        self
    }

    pub fn create(&self) {
        let mut popup = self.clone();

        unsafe {
            thread::spawn(move || {
                let instance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());
                let brush = CreateSolidBrush(CONFIG.lock().unwrap().app_bar_color as u32);
                let name = CString::new(popup.name.clone()).unwrap();
                let display = get_primary_display();

                let class = WNDCLASSA {
                    hInstance: instance as HINSTANCE,
                    lpszClassName: name.as_ptr(),
                    lpfnWndProc: Some(window_cb),
                    hbrBackground: brush,
                    ..WNDCLASSA::default()
                };

                RegisterClassA(&class);

                let window_handle = winapi::um::winuser::CreateWindowExA(
                    winapi::um::winuser::WS_EX_NOACTIVATE | winapi::um::winuser::WS_EX_TOPMOST,
                    name.as_ptr(),
                    name.as_ptr(),
                    winapi::um::winuser::WS_POPUPWINDOW,
                    display.width() / 2 - popup.width / 2,
                    display.height() / 2 - popup.height / 2,
                    popup.width,
                    popup.height,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    instance as HINSTANCE,
                    std::ptr::null_mut(),
                );

                popup.window = Window {
                    id: window_handle as i32,
                    title: popup.name.clone(),
                    ..Default::default()
                };

                *POPUP.lock().unwrap() = Some(popup);

                ShowWindow(window_handle, SW_SHOW);

                message_loop::start(|_| true);
            });
        }
    }
}

unsafe extern "system" fn window_cb(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_SETCURSOR {
        SetCursor(LoadCursorA(std::ptr::null_mut(), IDC_ARROW as *const i8));
    } else if msg == WM_PAINT {
        let hdc = GetDC(hwnd);
        let popup = POPUP.lock().unwrap().clone().unwrap();
        let c_text = util::to_widestring(&popup.text.join("\n"));
        let mut paint = PAINTSTRUCT::default();
        let mut rect = RECT::default();

        rect.top = 20;
        rect.right = popup.width;
        rect.bottom = popup.height;

        BeginPaint(hwnd, &mut paint);

        println!("Paint");
        SetTextColor(hdc, 0x00ffffff);
        SetBkColor(hdc, CONFIG.lock().unwrap().app_bar_color as u32);
        DrawTextW(hdc, c_text.as_ptr(), -1, &mut rect, DT_CENTER);

        ReleaseDC(hwnd, hdc);
        EndPaint(hwnd, &paint);
    }
    DefWindowProcA(hwnd, msg, w_param, l_param)
}

pub fn close() {
    let mut popup_guard = POPUP.lock().unwrap();
    if let Some(popup) = popup_guard.clone() {
        popup.window.send_close();
        *popup_guard = None;
    }
}

pub fn is_visible() -> bool {
    POPUP.lock().unwrap().is_some()
}
