use super::{SystemResult, SystemError, WindowId};
use thiserror::Error;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::*;
use crate::window::gwl_style::GwlStyle;
use crate::window::gwl_ex_style::GwlExStyle;
use crate::Rule;

#[derive(Error, Debug)]
pub enum WinError {
    #[error("Winapi return value is null")]
    Null,
    #[error("Winapi return value is falsy `${0}`")]
    Bool(i32)
}

type WinResult<T = ()> = Result<T, WinError>;

#[derive(Default, Debug, Copy, Clone)]
pub struct Rectangle {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

impl Rectangle {
    pub fn from_winapi_rect(rect: RECT) -> Self {
        Self {
            left: rect.left,
            right: rect.right,
            top: rect.top,
            bottom: rect.bottom
        }
    }
}

fn bool_to_result(v: BOOL) -> WinResult {
    if v == 0 {
        Ok(())
    } else {
        Err(WinError::Bool(v))
    }
}

fn nullable_to_result<T: PartialEq<i32>>(v: T) -> WinResult<T> {
    if v != 0 {
        Ok(v)
    } else {
        Err(WinError::Null)
    }
}

fn lresult_to_result(v: LRESULT) -> WinResult<LRESULT> {
    Ok(v)
}

mod api {
}

#[derive(Debug, Clone)]
pub struct WinWindow {
    pub id: WindowId,
    pub title: String,
    pub maximized: bool,
    pub rule: Option<Rule>,
    pub style: GwlStyle,
    pub exstyle: GwlExStyle,
    pub original_style: GwlStyle,
    pub original_rect: Rectangle,
}

impl WinWindow {
    pub fn remove_title_bar(&mut self) {
    }
    pub fn get_style(&self) -> SpecificResult<GwlStyle> {
        unsafe {
            nullable_to_result(GetWindowLongA(self.id as HWND, GWL_STYLE))
                .map(|x| GwlStyle::from_bits_unchecked(x as u32 as i32))
        }
    }
    pub fn get_ex_style(&self) -> SpecificResult<GwlExStyle> {
        unsafe {
            nullable_to_result(GetWindowLongA(self.id as HWND, GWL_EXSTYLE))
                .map(|x| GwlExStyle::from_bits_unchecked(x as u32 as i32))
        }
    }
    pub fn get_rect(&self) -> SpecificResult<Rectangle> {
        unsafe {
            let mut temp = RECT::default();
            nullable_to_result(GetWindowRect(self.id as HWND, &mut temp))
                .map(|_| Rectangle::from_winapi_rect(temp))
        }
    }
    pub fn reset_style(&mut self) {
        self.style = self.original_style;
    }
    pub fn update_style(&self) -> WinResult<i32> {
        unsafe {
            nullable_to_result::<i32>(winuser::SetWindowLongA(self.id as HWND, winuser::GWL_STYLE, self.style.bits()))
        }
    }
    /// TODO: extract somewhere else
    /// TODO: rewrite
    pub fn calculate_window_rect(
        &self,
        display: &Display,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> RECT {
        let rule = self.rule.clone().unwrap_or_default();
        let (display_app_bar, remove_title_bar, bar_height, use_border) = {
            let config = CONFIG.lock();

            (
                config.display_app_bar,
                config.remove_title_bar,
                config.bar.height,
                config.use_border,
            )
        };

        let mut left = x;
        let mut right = x + width;
        let mut top = y;
        let mut bottom = y + height;

        unsafe {
            let border_width = GetSystemMetricsForDpi(SM_CXFRAME, display.dpi);
            let border_height = GetSystemMetricsForDpi(SM_CYFRAME, display.dpi);

            if rule.chromium || rule.firefox || !remove_title_bar {
                let caption_height = GetSystemMetricsForDpi(SM_CYCAPTION, display.dpi);
                top += caption_height;
            } else {
                top -= border_height * 2;

                if use_border {
                    left += 1;
                    right -= 1;
                    top += 1;
                    bottom -= 1;
                }
            }

            if display_app_bar {
                top += bar_height;
                bottom += bar_height;
            }

            if rule.firefox || rule.chromium || (!remove_title_bar && rule.has_custom_titlebar) {
                if rule.firefox {
                    left -= (border_width as f32 * 1.5) as i32;
                    right += (border_width as f32 * 1.5) as i32;
                    bottom += (border_height as f32 * 1.5) as i32;
                } else if rule.chromium {
                    top -= border_height / 2;
                    left -= border_width * 2;
                    right += border_width * 2;
                    bottom += border_height * 2;
                }
                left += border_width * 2;
                right -= border_width * 2;
                top += border_height * 2;
                bottom -= border_height * 2;
            } else {
                top += border_height * 2;
            }
        }

        let mut rect = RECT {
            left,
            right,
            top,
            bottom,
        };

        //println!("before {}", rect_to_string(rect));

        unsafe {
            AdjustWindowRectEx(
                &mut rect,
                self.style.bits() as u32,
                0,
                self.exstyle.bits() as u32,
            );
        }

        // println!("after {}", rect_to_string(rect));

        rect
    }
    /// TODO: rewrite
    pub fn to_foreground(&self, topmost: bool) -> Result<(), util::WinApiResultError> {
        unsafe {
            util::winapi_nullable_to_result(SetWindowPos(
                self.id as HWND,
                if topmost { HWND_TOPMOST } else { HWND_TOP },
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
            ))?;
        }

        Ok(())
    }
    /// TODO: rewrite
    pub fn remove_topmost(&self) -> Result<(), util::WinApiResultError> {
        unsafe {
            nullable_to_result(self.set_window_pos(Rectangle::default())
                self.id as HWND,
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
            ))
        }
    }
    pub fn set_window_pos(&self, rect: Rectangle) -> WinResult {
        unsafe {
            bool_to_result(winuser::SetWindowPos(
                self.id as HWND,
                std::ptr::null_mut(),
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                0,
            ))
        }
    }
    pub fn reset_pos(&self) -> WinResult {
        self.set_window_pos(self.original_rect)
    }
    pub fn new() -> Self {
        Self {
            id: 0,
            title: String::from(""),
            maximized: false,
            rule: None,
            style: GwlStyle::default(),
            exstyle: GwlExStyle::default(),
            original_style: GwlStyle::default(),
            original_rect: Rectangle::default(),
        }
    }
    pub fn cleanup(&mut self) -> SystemResult { 
        self.reset_style();
        self.update_style().map_err(SystemError::CleanupWindow)?;
        self.reset_pos().map_err(SystemError::CleanupWindow)?;

        if self.maximized {
            self.maximize();
        }

        Ok(())
    }
    pub fn show(&self) -> SystemResult {
        unsafe {
            bool_to_result(winuser::ShowWindow(self.id as HWND, winuser::SW_SHOW))
                .map(|_| {})
                .map_err(SystemError::ShowWindow)
        }
    }
    pub fn hide(&self) -> SystemResult {
        unsafe {
            bool_to_result(winuser::ShowWindow(self.id as HWND, winuser::SW_HIDE))
                .map(|_| {})
                .map_err(SystemError::HideWindow)
        }
    }
    pub fn close(&self) -> SystemResult {
        unsafe {
            lresult_to_result(winuser::SendMessageA(self.id as HWND, winuser::WM_SYSCOMMAND, winuser::SC_CLOSE, 0))
                .map(|_| {})
                .map_err(SystemError::CloseWindow)
        }
    }
    pub fn focus(&self) -> SystemResult {
        unsafe {
            bool_to_result(winuser::SetForegroundWindow(self.id as HWND))
                .map(|_| {})
                .map_err(SystemError::FocusWindow)
        }
    }
    pub fn minimize(&self) -> SystemResult {
        unsafe {
            lresult_to_result(winuser::SendMessageA(self.id as HWND, winuser::WM_SYSCOMMAND, winuser::SC_MINIMIZE, 0))
                .map(|_| {})
                .map_err(SystemError::MinimizeWindow)
        }
    }
    pub fn maximize(&self) -> SystemResult {
        unsafe {
            lresult_to_result(winuser::SendMessageA(self.id as HWND, winuser::WM_SYSCOMMAND, winuser::SC_MAXIMIZE, 0))
                .map(|_| {})
                .map_err(SystemError::MaximizeWindow)
        }
    }
}
