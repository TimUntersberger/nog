use winapi::shared::minwindef::BOOL;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::winuser::GetForegroundWindow;
use winapi::um::winuser::GetParent;
use winapi::um::winuser::GetWindowLongA;
use winapi::um::winuser::GetWindowRect;
use winapi::um::winuser::SendMessageA;
use winapi::um::winuser::SetForegroundWindow;
use winapi::um::winuser::SetWindowLongA;
use winapi::um::winuser::SetWindowPos;
use winapi::um::winuser::ShowWindow;
use winapi::um::winuser::GWL_STYLE;
use winapi::um::winuser::SW_HIDE;
use winapi::um::winuser::SW_SHOW;
use winapi::um::winuser::WM_DESTROY;
use winapi::um::winuser::WS_BORDER;
use winapi::um::winuser::WS_CAPTION;
use winapi::um::winuser::WS_THICKFRAME;

use crate::util;

#[derive(Clone)]
pub struct Window {
    pub id: i32,
    pub name: String,
    pub original_style: i32,
    pub original_rect: RECT,
}

impl Window {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: String::from(""),
            original_style: 0,
            original_rect: RECT {
                left: 0,
                right: 0,
                top: 0,
                bottom: 0,
            },
        }
    }
    pub fn reset_style(&self) -> Result<(), util::WinApiResultError> {
        unsafe {
            util::winapi_nullable_to_result(SetWindowLongA(
                self.id as HWND,
                GWL_STYLE,
                self.original_style,
            ))?;
        }

        Ok(())
    }
    pub fn reset_pos(&self) -> Result<(), util::WinApiResultError> {
        unsafe {
            util::winapi_nullable_to_result(SetWindowPos(
                self.id as HWND,
                std::ptr::null_mut(),
                self.original_rect.left,
                self.original_rect.top,
                self.original_rect.right - self.original_rect.left,
                self.original_rect.bottom - self.original_rect.top,
                0,
            ))?;
        }

        Ok(())
    }
    pub fn get_foreground_window() -> Result<HWND, util::WinApiResultError> {
        unsafe { util::winapi_ptr_to_result(GetForegroundWindow()) }
    }
    pub fn get_parent_window(&self) -> Result<HWND, util::WinApiResultError> {
        unsafe { util::winapi_ptr_to_result(GetParent(self.id as HWND)) }
    }
    pub fn get_style(&self) -> Result<i32, util::WinApiResultError> {
        unsafe { util::winapi_nullable_to_result(GetWindowLongA(self.id as HWND, GWL_STYLE)) }
    }
    pub fn get_rect(&self) -> Result<RECT, util::WinApiResultError> {
        unsafe {
            let mut temp = RECT::default();
            util::winapi_nullable_to_result(GetWindowRect(self.id as HWND, &mut temp))?;
            Ok(temp)
        }
    }
    pub fn show(&self) -> util::WinApiResult<BOOL> {
        unsafe {
            return util::winapi_err_to_result(ShowWindow(self.id as HWND, SW_SHOW));
        }
    }
    pub fn hide(&self) -> util::WinApiResult<BOOL> {
        unsafe {
            return util::winapi_err_to_result(ShowWindow(self.id as HWND, SW_HIDE));
        }
    }
    pub fn to_foreground(&self) -> Result<(), util::WinApiResultError> {
        unsafe {
            util::winapi_nullable_to_result(SetForegroundWindow(self.id as HWND))?;
        }

        Ok(())
    }
    pub fn send_destroy(&self) {
        unsafe {
            //TODO: Handle Error
            SendMessageA(self.id as HWND, WM_DESTROY, 0, 0);
        }
    }
    pub fn remove_title_bar(&self) -> Result<i32, util::WinApiResultError> {
        unsafe {
            util::winapi_nullable_to_result(SetWindowLongA(
                self.id as HWND,
                GWL_STYLE,
                self.original_style & !WS_CAPTION as i32 & !WS_THICKFRAME as i32 | WS_BORDER as i32,
            ))
        }
    }
}
