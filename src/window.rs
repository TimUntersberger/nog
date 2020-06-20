pub mod gwl_style;
pub mod gwl_ex_style;

use crate::config::Rule;
use winapi::um::winuser::WM_CLOSE;
use winapi::um::winuser::GWL_EXSTYLE;
use winapi::um::winuser::HWND_NOTOPMOST;
use winapi::um::winuser::HWND_TOPMOST;
use winapi::um::winuser::SWP_NOMOVE;
use winapi::um::winuser::SWP_NOSIZE;
use winapi::um::winuser::HWND_TOP;
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

use winapi::shared::minwindef::BOOL;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;

use crate::util;
use gwl_style::GwlStyle;
use gwl_ex_style::GwlExStyle;

#[derive(Clone)]
pub struct Window {
    pub id: i32,
    pub title: String,
    pub rule: Option<Rule>,
    pub original_style: GwlStyle,
    pub original_rect: RECT,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::from(""),
            rule: None,
            original_style: GwlStyle::default(),
            original_rect: RECT::default(),
        }
    }
}

impl Window {
    pub fn reset_style(&self) -> Result<(), util::WinApiResultError> {
        unsafe {
            util::winapi_nullable_to_result(SetWindowLongA(
                self.id as HWND,
                GWL_STYLE,
                self.original_style.bits(),
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
    pub fn get_style(&self) -> Result<GwlStyle, util::WinApiResultError> {
        unsafe { 
            let bits = util::winapi_nullable_to_result(GetWindowLongA(self.id as HWND, GWL_STYLE))?;
            return Ok(GwlStyle::from_bits_unchecked(bits as u32 as i32));
        }
    } 
    pub fn get_ex_style(&self) -> Result<GwlExStyle, util::WinApiResultError> {
        unsafe { 
            let bits = util::winapi_nullable_to_result(GetWindowLongA(self.id as HWND, GWL_EXSTYLE))?;
            return Ok(GwlExStyle::from_bits_unchecked(bits as u32 as i32));
        }
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
    pub fn to_foreground(&self, topmost: bool) -> Result<(), util::WinApiResultError> {
        unsafe {
            util::winapi_nullable_to_result(SetWindowPos(
                self.id as HWND,
                if topmost { HWND_TOPMOST } else { HWND_TOP },
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE
            ))?;
        }

        Ok(())
    }  
    pub fn remove_topmost(&self) -> Result<(), util::WinApiResultError> {
        unsafe {
            util::winapi_nullable_to_result(SetWindowPos(
                self.id as HWND,
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE
            ))?;
        }

        Ok(())
    }
    /**
     * This also brings the window to the foreground
     */
    pub fn focus(&self) -> Result<(), util::WinApiResultError> {
        unsafe {
            util::winapi_nullable_to_result(SetForegroundWindow(self.id as HWND))?;
        }

        Ok(())
    }
    pub fn send_close(&self) {
        unsafe {
            //TODO: Handle Error
            SendMessageA(self.id as HWND, WM_CLOSE, 0, 0);
        }
    }
    pub fn remove_title_bar(&self) -> Result<i32, util::WinApiResultError> {
        let mut new_style = self.original_style.clone();

        new_style.remove(GwlStyle::CAPTION);

        if let Some(rule) = &self.rule {
            if !rule.has_custom_titlebar {
                new_style.remove(GwlStyle::THICKFRAME);
            }
        }

        new_style.insert(GwlStyle::BORDER);

        unsafe {
            util::winapi_nullable_to_result(SetWindowLongA(
                self.id as HWND,
                GWL_STYLE,
                new_style.bits(),
            ))
        }
    }
}
