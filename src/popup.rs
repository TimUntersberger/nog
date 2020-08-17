use crate::{bar, display::get_primary_display, message_loop, util, window::Window, CONFIG};

use std::{
    ffi::CString,
    sync::{Arc, Mutex},
    thread,
};
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::wingdi::SetBkColor;
use winapi::um::wingdi::SetTextColor;

use winapi::um::winuser::{
    BeginPaint, DefWindowProcA, EndPaint, GetDC, LoadCursorA, RegisterClassA, ReleaseDC, SetCursor,
    ShowWindow, IDC_ARROW, PAINTSTRUCT, SW_SHOW, WM_PAINT, WM_SETCURSOR,
};
use winapi::{
    shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM},
    um::winuser::{DrawTextW, SetWindowPos, UnregisterClassA, DT_CALCRECT, WM_CLOSE, WNDCLASSA},
};

use lazy_static::lazy_static;

lazy_static! {
    static ref POPUP: Mutex<Option<Popup>> = Mutex::new(None);
}

pub type PopupActionCallback = Arc<dyn Fn() -> () + Sync + Send>;
#[derive(Default, Clone)]
pub struct PopupAction {
    pub text: String,
    pub cb: Option<PopupActionCallback>,
}

#[derive(Clone)]
pub struct Popup {
    window: Window,
    width: i32,
    padding: i32,
    height: i32,
    text: Vec<String>,
    pub actions: Vec<PopupAction>,
}

impl Popup {
    pub fn new() -> Self {
        Self {
            window: Window::default(),
            width: 0,
            height: 0,
            padding: 5,
            text: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn with_text(&mut self, text: &[&str]) -> &mut Self {
        self.text = text.iter().map(|x| x.to_string()).collect();
        self
    }

    pub fn with_padding(&mut self, padding: i32) -> &mut Self {
        self.padding = padding + 5;
        self
    }

    /// Creates the window for the popup with the configured parameters.
    ///
    /// This function closes a popup that is currently visible.
    pub fn create(&self) {
        if is_visible() {
            close();
        }

        let mut popup = self.clone();

        unsafe {
            thread::spawn(move || {
                let instance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());
                let name = CString::new("NogPopup").unwrap();
                let display = get_primary_display();

                let window_handle = winapi::um::winuser::CreateWindowExA(
                    winapi::um::winuser::WS_EX_NOACTIVATE | winapi::um::winuser::WS_EX_TOPMOST,
                    name.as_ptr(),
                    name.as_ptr(),
                    winapi::um::winuser::WS_POPUPWINDOW,
                    0,
                    0,
                    0,
                    0,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    instance as HINSTANCE,
                    std::ptr::null_mut(),
                );

                let mut rect = RECT::default();
                let hdc = GetDC(window_handle);
                let c_text = util::to_widestring(&popup.text.join("\n"));

                bar::font::set_font(hdc);
                DrawTextW(hdc, c_text.as_ptr(), -1, &mut rect, DT_CALCRECT);

                let width = rect.right - rect.left;
                let height = rect.bottom - rect.top;

                let x = display.width() / 2 - width / 2 - popup.padding;
                let y = display.height() / 2 - height / 2 - popup.padding;

                SetWindowPos(
                    window_handle,
                    std::ptr::null_mut(),
                    x,
                    y,
                    width + popup.padding * 2,
                    height + popup.padding * 2,
                    0,
                );

                popup.width = width;
                popup.height = height;
                popup.window = Window {
                    id: window_handle as i32,
                    title: "NogPopup".into(),
                    ..Default::default()
                };

                *POPUP.lock().unwrap() = Some(popup);

                ShowWindow(window_handle, SW_SHOW);

                message_loop::start(|_| true);
            });
        }
    }
}

pub fn init() {
    bar::font::load_font();
    unsafe {
        let instance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());
        let brush = CreateSolidBrush(CONFIG.lock().unwrap().bar.color as u32);
        let name = CString::new("NogPopup").unwrap();

        let class = WNDCLASSA {
            hInstance: instance as HINSTANCE,
            lpszClassName: name.as_ptr(),
            lpfnWndProc: Some(window_cb),
            hbrBackground: brush,
            ..WNDCLASSA::default()
        };

        RegisterClassA(&class);
    }
}

pub fn cleanup() {
    close();
    let name = CString::new("NogPopup").unwrap();
    unsafe {
        UnregisterClassA(
            name.as_ptr(),
            winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut()),
        );
    }
}

unsafe extern "system" fn window_cb(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_CLOSE {
        let popup = POPUP.lock().unwrap().clone().unwrap();
        for action in popup.actions {
            let cb = action.cb.unwrap().clone();
            cb();
        }
        *POPUP.lock().unwrap() = None;
    } else if msg == WM_SETCURSOR {
        SetCursor(LoadCursorA(std::ptr::null_mut(), IDC_ARROW as *const i8));
    } else if msg == WM_PAINT {
        let popup = POPUP.lock().unwrap().clone().unwrap();
        let mut rect = RECT::default();

        rect.right = popup.width;
        rect.bottom = popup.height;

        rect.left += popup.padding;
        rect.top += popup.padding;
        rect.right += popup.padding;
        rect.bottom += popup.padding;

        let mut paint = PAINTSTRUCT::default();
        BeginPaint(hwnd, &mut paint);

        let hdc = GetDC(hwnd);
        bar::font::set_font(hdc);
        SetTextColor(hdc, 0x00ffffff);
        SetBkColor(hdc, CONFIG.lock().unwrap().bar.color as u32);

        let c_text = util::to_widestring(&popup.text.join("\n"));
        DrawTextW(hdc, c_text.as_ptr(), -1, &mut rect, 0);

        ReleaseDC(hwnd, hdc);

        EndPaint(hwnd, &paint);
    }
    DefWindowProcA(hwnd, msg, w_param, l_param)
}

/// Close the current popup, if there is one.
pub fn close() {
    let maybe_window = POPUP.lock().unwrap().clone().map(|p| p.window);
    if let Some(window) = maybe_window {
        window.close();
    }
}

/// Is there a popup currently visible?
pub fn is_visible() -> bool {
    POPUP.lock().unwrap().is_some()
}
