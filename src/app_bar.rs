use crate::display::Display;
use log::debug;
use std::sync::Mutex;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HBRUSH;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::wingdi::SetBkMode;
use winapi::um::wingdi::SetTextColor;
use winapi::um::wingdi::TRANSPARENT;
use winapi::um::winuser::BeginPaint;
use winapi::um::winuser::DefWindowProcA;
use winapi::um::winuser::DrawTextA;
use winapi::um::winuser::EndPaint;
use winapi::um::winuser::FillRect;
use winapi::um::winuser::GetClientRect;
use winapi::um::winuser::GetDC;
use winapi::um::winuser::RegisterClassA;
use winapi::um::winuser::ShowWindow;
use winapi::um::winuser::DT_CENTER;
use winapi::um::winuser::DT_SINGLELINE;
use winapi::um::winuser::DT_VCENTER;
use winapi::um::winuser::PAINTSTRUCT;
use winapi::um::winuser::SW_SHOW;
use winapi::um::winuser::WM_PAINT;
use winapi::um::winuser::WNDCLASSA;
use lazy_static::lazy_static;

use std::ffi::CString;

use crate::util;
use crate::CONFIG;

lazy_static! {
    pub static ref HEIGHT: Mutex<i32> = Mutex::new(0);
    pub static ref WINDOW: Mutex<i32> = Mutex::new(0);
}

unsafe extern "system" fn window_cb(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let window = *WINDOW.lock().unwrap();

    if msg == WM_PAINT && window != 0 {
        let mut paint = PAINTSTRUCT::default();

        GetClientRect(window as HWND, &mut paint.rcPaint);

        BeginPaint(window as HWND, &mut paint);
        EndPaint(window as HWND, &paint);
    }

    return DefWindowProcA(hwnd, msg, w_param, l_param);
}

pub fn create(display: &Display) -> Result<(), util::WinApiResultError> {
    let name = "wwm_app_bar";
    let mut height_guard = HEIGHT.lock().unwrap();
    *height_guard = 20;

    unsafe {
        let instance = util::winapi_nullable_to_result(
            winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut()) as i32,
        )?;
        let background_brush =
            util::winapi_nullable_to_result(CreateSolidBrush(CONFIG.app_bar_bg as u32) as i32)?;

        let class = WNDCLASSA {
            hInstance: instance as HINSTANCE,
            lpszClassName: name.as_ptr() as *const i8,
            lpfnWndProc: Some(window_cb),
            hbrBackground: background_brush as HBRUSH,
            ..WNDCLASSA::default()
        };

        RegisterClassA(&class);

        let window_handle = util::winapi_ptr_to_result(winapi::um::winuser::CreateWindowExA(
            winapi::um::winuser::WS_EX_NOACTIVATE | winapi::um::winuser::WS_EX_TOPMOST,
            name.as_ptr() as *const i8,
            name.as_ptr() as *const i8,
            winapi::um::winuser::WS_POPUPWINDOW & !winapi::um::winuser::WS_BORDER,
            0,
            0,
            display.width,
            *height_guard,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance as HINSTANCE,
            std::ptr::null_mut(),
        ))?;

        *WINDOW.lock().unwrap() = window_handle as i32;

        ShowWindow(window_handle, SW_SHOW);
    }

    Ok(())
}

pub fn clear() {
    let mut rect = RECT::default();
    let window = *WINDOW.lock().unwrap();

    unsafe {
        let brush = CreateSolidBrush(CONFIG.app_bar_bg as u32);
        let dc = GetDC(window as HWND);

        GetClientRect(window as HWND, &mut rect);
        FillRect(dc, &mut rect, brush);
    }
}

pub fn draw_workspace(idx: i32, id: i32, focused: bool) -> Result<(), util::WinApiResultError> {
    let window = *WINDOW.lock().unwrap() as HWND;
    if window != std::ptr::null_mut() {
        let mut rect = RECT::default();
        let height = *HEIGHT.lock().unwrap();

        unsafe {
            debug!("Getting the rect for the appbar");
            util::winapi_nullable_to_result(GetClientRect(window, &mut rect))?;

            rect.left = rect.left + height * idx;
            rect.right = rect.left + height;

            debug!("Getting the device context");
            let hdc = util::winapi_ptr_to_result(GetDC(window))?;

            debug!("Creating brush for background");
            let brush = if focused {
                util::winapi_ptr_to_result(CreateSolidBrush(0x00ffffff))?
            } else {
                util::winapi_ptr_to_result(CreateSolidBrush(CONFIG.app_bar_workspace_bg as u32))?
            };

            debug!("Drawing the background");
            util::winapi_nullable_to_result(FillRect(hdc, &mut rect, brush))?;

            debug!("Setting the background color to transparent");
            util::winapi_nullable_to_result(SetBkMode(hdc, TRANSPARENT as i32))?;

            debug!("Setting the text color");
            //TODO: handle error
            if focused {
                SetTextColor(hdc, CONFIG.app_bar_workspace_bg as u32);
            } else {
                SetTextColor(hdc, 0x00ffffff);
            }

            let id_str = id.to_string();
            let len = id_str.len() as i32;
            let id_cstr = CString::new(id_str).unwrap();

            debug!("Writing the text");
            util::winapi_nullable_to_result(DrawTextA(
                hdc,
                id_cstr.as_ptr(),
                len,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            ))?;
        }
    }

    Ok(())
}
