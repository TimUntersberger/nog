use crate::{
    display::get_primary_display, event::Event, message_loop, task_bar::HEIGHT, util, CHANNEL,
    CONFIG, DISPLAYS,
};
use log::{debug, error, info};
use std::{sync::Mutex, ffi::CString, collections::HashMap};
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
    um::winuser::{DrawTextW, WNDCLASSA, DT_CENTER, DT_VCENTER, DT_SINGLELINE},
};
use lazy_static::lazy_static;

lazy_static! {
    static ref POPUPS: Mutex<HashMap<i32, Popup>> = Mutex::new(HashMap::new());
}

#[derive(Clone, Debug)]
pub struct Popup {
    pub name: String,
    pub width: i32,
    pub height: i32,
}

impl Popup {
    pub fn new(name: String, width: i32, height: i32) -> Self {
        Self {
            name,
            width,
            height,
        }
    }

    pub fn create(&self) {
        unsafe {
            let instance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());
            let brush = CreateSolidBrush(CONFIG.lock().unwrap().app_bar_color as u32);
            let name = CString::new(self.name.clone()).unwrap();
            let display = get_primary_display();

            let class = WNDCLASSA {
                hInstance: instance as HINSTANCE,
                lpszClassName: name.as_ptr(),
                lpfnWndProc: Some(window_cb),
                hbrBackground: brush,
                ..WNDCLASSA::default()
            };

            RegisterClassA(&class);

            //TODO: handle error
            let window_handle = winapi::um::winuser::CreateWindowExA(
                winapi::um::winuser::WS_EX_NOACTIVATE | winapi::um::winuser::WS_EX_TOPMOST,
                name.as_ptr(),
                name.as_ptr(),
                winapi::um::winuser::WS_POPUPWINDOW,
                display.width() / 2 - self.width / 2,
                display.height() / 2 - self.height / 2,
                self.width,
                self.height,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance as HINSTANCE,
                std::ptr::null_mut(),
            );

            POPUPS.lock().unwrap().insert(window_handle as i32, self.clone());

            ShowWindow(window_handle, SW_SHOW);

            message_loop::start(|_| true);
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
        let c_text = util::to_widestring("Hello World".into());
        let popups = POPUPS.lock().unwrap();
        let popup = popups.get(&(hwnd as i32)).unwrap();
        let mut paint = PAINTSTRUCT::default();
        let mut rect = RECT::default();

        rect.right = popup.width;
        rect.bottom = popup.height;

        BeginPaint(hwnd, &mut paint);

        println!("Paint");
        SetTextColor(hdc, 0x00ffffff);
        SetBkColor(hdc, CONFIG.lock().unwrap().app_bar_color as u32);
        DrawTextW(hdc, c_text.as_ptr(), -1, &mut rect, DT_VCENTER | DT_CENTER | DT_SINGLELINE);

        ReleaseDC(hwnd, hdc);
        EndPaint(hwnd, &paint);
    }
    DefWindowProcA(hwnd, msg, w_param, l_param)
}
