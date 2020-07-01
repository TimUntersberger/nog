use crate::config::Rule;
use crate::util;
use crate::util::rect_to_string;
use crate::CONFIG;
use gwl_ex_style::GwlExStyle;
use gwl_style::GwlStyle;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::winuser::AdjustWindowRectEx;
use winapi::um::winuser::GetForegroundWindow;
use winapi::um::winuser::GetParent;
use winapi::um::winuser::GetSystemMetrics;
use winapi::um::winuser::GetWindowLongA;
use winapi::um::winuser::GetWindowRect;
use winapi::um::winuser::SendMessageA;
use winapi::um::winuser::SetForegroundWindow;
use winapi::um::winuser::SetWindowLongA;
use winapi::um::winuser::SetWindowPos;
use winapi::um::winuser::ShowWindow;
use winapi::um::winuser::GWL_EXSTYLE;
use winapi::um::winuser::GWL_STYLE;
use winapi::um::winuser::HWND_NOTOPMOST;
use winapi::um::winuser::HWND_TOP;
use winapi::um::winuser::HWND_TOPMOST;
use winapi::um::winuser::SM_CXFRAME;
use winapi::um::winuser::SM_CYCAPTION;
use winapi::um::winuser::SM_CYFRAME;
use winapi::um::winuser::SWP_NOMOVE;
use winapi::um::winuser::SWP_NOSIZE;
use winapi::um::winuser::SW_HIDE;
use winapi::um::winuser::SW_SHOW;
use winapi::um::winuser::WM_CLOSE;

pub mod gwl_ex_style;
pub mod gwl_style;

#[derive(Clone)]
pub struct Window {
    pub id: i32,
    pub title: String,
    pub rule: Option<Rule>,
    pub style: GwlStyle,
    pub exstyle: GwlExStyle,
    pub original_style: GwlStyle,
    pub original_rect: RECT,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::from(""),
            rule: None,
            style: GwlStyle::default(),
            exstyle: GwlExStyle::default(),
            original_style: GwlStyle::default(),
            original_rect: RECT::default(),
        }
    }
}

impl Window {
    pub fn reset_style(&mut self) -> Result<(), util::WinApiResultError> {
        self.style = self.original_style;

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
            Ok(GwlStyle::from_bits_unchecked(bits as u32 as i32))
        }
    }
    pub fn get_ex_style(&self) -> Result<GwlExStyle, util::WinApiResultError> {
        unsafe {
            let bits =
                util::winapi_nullable_to_result(GetWindowLongA(self.id as HWND, GWL_EXSTYLE))?;
            Ok(GwlExStyle::from_bits_unchecked(bits as u32 as i32))
        }
    }
    pub fn get_rect(&self) -> Result<RECT, util::WinApiResultError> {
        unsafe {
            let mut temp = RECT::default();
            util::winapi_nullable_to_result(GetWindowRect(self.id as HWND, &mut temp))?;
            Ok(temp)
        }
    }
    pub fn show(&self) {
        unsafe {
            ShowWindow(self.id as HWND, SW_SHOW);
        }
    }
    pub fn hide(&self) {
        unsafe {
            ShowWindow(self.id as HWND, SW_HIDE);
        }
    }
    pub fn calculate_window_rect(&self, x: i32, y: i32, width: i32, height: i32) -> RECT {
        let rule = self.rule.clone().unwrap_or_default();
        let (display_app_bar, remove_title_bar, app_bar_height) = {
            let config = CONFIG.lock().unwrap();

            (
                config.display_app_bar,
                config.remove_title_bar,
                config.app_bar_height,
            )
        };

        let mut left = x;
        let mut right = x + width;
        let mut top = y;
        let mut bottom = y + height;

        unsafe {
            let border_width = GetSystemMetrics(SM_CXFRAME);
            let border_height = GetSystemMetrics(SM_CYFRAME);

            if rule.chromium || rule.firefox || !remove_title_bar {
                let caption_height = GetSystemMetrics(SM_CYCAPTION);
                top += caption_height;
            } else {
                top -= border_height * 2;
                bottom -= border_height / 2;

                left += 1;
                right -= 1;
                top += 1;
                bottom += 1;
            }

            // if !remove_task_bar {
            //     bottom -= *task_bar::HEIGHT.lock().unwrap();
            // }

            if display_app_bar {
                top += app_bar_height;
                bottom += app_bar_height;
            }

            if rule.firefox || rule.chromium || (!remove_title_bar && rule.has_custom_titlebar) {
                // looks like the frame around firefox is smaller than chrome's frame by about 2 pixels
                // I don't see any other window that behaves like these two pieces of shit

                if rule.firefox {
                    left -= (border_width as f32 * 1.5) as i32;
                    right += (border_width as f32 * 1.5) as i32;
                    bottom += (border_height as f32 * 1.5) as i32;
                } else if rule.chromium {
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
    pub fn remove_topmost(&self) -> Result<(), util::WinApiResultError> {
        unsafe {
            util::winapi_nullable_to_result(SetWindowPos(
                self.id as HWND,
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
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
    pub fn update_style(&self) {
        unsafe {
            SetWindowLongA(self.id as HWND, GWL_STYLE, self.style.bits());
        }
    }
    pub fn update_exstyle(&self) {
        unsafe {
            SetWindowLongA(self.id as HWND, GWL_EXSTYLE, self.exstyle.bits());
        }
    }
    pub fn remove_title_bar(&mut self) {
        let rule = self.rule.clone().unwrap_or_default();
        if !rule.chromium && !rule.firefox {
            self.style.remove(GwlStyle::CAPTION);
            self.style.remove(GwlStyle::THICKFRAME);
        }
        self.style.insert(GwlStyle::BORDER);
    }
}
