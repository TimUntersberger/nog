use super::{font::set_font, WINDOWS};
use crate::{display::get_display_by_hmonitor, util, CONFIG};
use std::ffi::CString;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::wingdi::{CreateSolidBrush, DeleteObject, SetBkColor, SetTextColor};
use winapi::um::winuser::{
    DrawTextA, FillRect, GetClientRect, GetDC, ReleaseDC, DT_CENTER, DT_END_ELLIPSIS,
    DT_SINGLELINE, DT_VCENTER,
};

pub fn draw(hwnd: HWND, mode: Option<String>) -> Result<(), util::WinApiResultError> {
    if !hwnd.is_null() {
        let mut rect = RECT::default();

        unsafe {
            util::winapi_nullable_to_result(GetClientRect(hwnd, &mut rect))?;

            let app_bar_bg = CONFIG.lock().unwrap().app_bar_bg as u32;
            let hmonitor = WINDOWS
                .lock()
                .unwrap()
                .iter()
                .find(|(_, v)| **v == hwnd as i32)
                .map(|(k, _)| *k)
                .expect("Failed to get current monitor");

            let display = get_display_by_hmonitor(hmonitor);

            let hdc = util::winapi_ptr_to_result(GetDC(hwnd))?;

            rect.left = display.width() / 4 * 3 - 200;
            rect.right = rect.left + 300;

            if let Some(text) = mode.map(|m| format!("{}", m)) {
                let text_len = text.len() as i32;
                let c_text = CString::new(text).unwrap();

                set_font(hdc);

                //TODO: handle error
                if CONFIG.lock().unwrap().light_theme {
                    SetTextColor(hdc, 0x00333333);
                } else {
                    SetTextColor(hdc, 0x00ffffff);
                }

                SetBkColor(hdc, app_bar_bg);

                util::winapi_nullable_to_result(DrawTextA(
                    hdc,
                    c_text.as_ptr(),
                    text_len,
                    &mut rect,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
                ))?;
            } else {
                let brush = CreateSolidBrush(app_bar_bg);

                FillRect(hdc, &rect, brush);

                DeleteObject(brush as *mut std::ffi::c_void);
            }

            ReleaseDC(hwnd, hdc);
        }
    }

    Ok(())
}
