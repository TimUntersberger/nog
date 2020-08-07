use super::font::set_font;
use crate::{task_bar::HEIGHT, util, CONFIG};
use std::ffi::CString;
use winapi::shared::windef::HWND;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::wingdi::DeleteObject;
use winapi::um::wingdi::SetBkMode;
use winapi::um::wingdi::SetTextColor;
use winapi::um::wingdi::TRANSPARENT;
use winapi::{
    shared::windef::RECT,
    um::winuser::{
        DrawTextA, FillRect, GetClientRect, GetDC, ReleaseDC, DT_CENTER, DT_SINGLELINE, DT_VCENTER,
    },
};

pub fn draw_workspace(
    hwnd: HWND,
    idx: i32,
    id: i32,
    focused: bool,
) -> Result<(), util::WinApiResultError> {
    if !hwnd.is_null() {
        let mut rect = RECT::default();
        let height = *HEIGHT.lock().unwrap();

        unsafe {
            util::winapi_nullable_to_result(GetClientRect(hwnd, &mut rect))?;
            rect.left += height * idx;
            rect.right = rect.left + height;

            let hdc = util::winapi_ptr_to_result(GetDC(hwnd))?;

            set_font(hdc);

            let app_bar_bg = CONFIG.lock().unwrap().app_bar_bg;

            SetBkMode(hdc, TRANSPARENT as i32);

            if CONFIG.lock().unwrap().light_theme {
                SetTextColor(hdc, 0x00333333);

                let brush = if focused {
                    CreateSolidBrush(util::scale_color(app_bar_bg, 0.75) as u32)
                } else {
                    CreateSolidBrush(util::scale_color(app_bar_bg, 0.9) as u32)
                };

                FillRect(hdc, &rect, brush);
                DeleteObject(brush as *mut std::ffi::c_void);
            } else {
                SetTextColor(hdc, 0x00ffffff);

                let brush = if focused {
                    CreateSolidBrush(util::scale_color(app_bar_bg, 2.0) as u32)
                } else {
                    CreateSolidBrush(util::scale_color(app_bar_bg, 1.5) as u32)
                };

                FillRect(hdc, &rect, brush);
                DeleteObject(brush as *mut std::ffi::c_void);
            }

            let id_str = id.to_string();
            let len = id_str.len() as i32;
            let id_cstr = CString::new(id_str).unwrap();

            util::winapi_nullable_to_result(DrawTextA(
                hdc,
                id_cstr.as_ptr(),
                len,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            ))?;

            ReleaseDC(hwnd, hdc);
        }
    }

    Ok(())
}
