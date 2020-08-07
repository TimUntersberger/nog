use super::{font::set_font, WINDOWS};
use crate::{display::get_display_by_hmonitor, util, CONFIG};
use std::ffi::CString;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::shared::windef::SIZE;
use winapi::um::wingdi::GetTextExtentPoint32A;
use winapi::um::wingdi::SetBkColor;
use winapi::um::wingdi::SetTextColor;
use winapi::um::winuser::{
    DrawTextA, GetClientRect, GetDC, ReleaseDC, DT_CENTER, DT_SINGLELINE, DT_VCENTER,
};

pub fn draw_datetime(hwnd: HWND) -> Result<(), util::WinApiResultError> {
    if !hwnd.is_null() {
        let mut rect = RECT::default();

        unsafe {
            util::winapi_nullable_to_result(GetClientRect(hwnd, &mut rect))?;
            let text = format!(
                "{}",
                chrono::Local::now().format(&CONFIG.lock().unwrap().app_bar_time_pattern)
            );
            let text_len = text.len() as i32;
            let c_text = CString::new(text).unwrap();
            let hmonitor = WINDOWS
                .lock()
                .unwrap()
                .iter()
                .find(|(_, v)| **v == hwnd as i32)
                .map(|(k, _)| *k)
                .expect("Failed to get current monitor");

            let display = get_display_by_hmonitor(hmonitor);

            let hdc = util::winapi_ptr_to_result(GetDC(hwnd))?;

            set_font(hdc);

            let mut size = SIZE::default();

            util::winapi_nullable_to_result(GetTextExtentPoint32A(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut size,
            ))?;

            rect.left = display.width() / 2 - (size.cx / 2) - 10;
            rect.right = display.width() / 2 + (size.cx / 2) + 10;

            //TODO: handle error
            if CONFIG.lock().unwrap().light_theme {
                SetTextColor(hdc, 0x00333333);
            } else {
                SetTextColor(hdc, 0x00ffffff);
            }

            SetBkColor(hdc, CONFIG.lock().unwrap().app_bar_bg as u32);

            util::winapi_nullable_to_result(DrawTextA(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            ))?;

            let text = format!(
                "{}",
                chrono::Local::now().format(&CONFIG.lock().unwrap().app_bar_date_pattern)
            );
            let text_len = text.len() as i32;
            let c_text = CString::new(text).unwrap();

            util::winapi_nullable_to_result(GetTextExtentPoint32A(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut size,
            ))?;

            rect.right = display.width() - 10;
            rect.left = rect.right - size.cx;

            util::winapi_nullable_to_result(DrawTextA(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            ))?;

            ReleaseDC(hwnd, hdc);
        }
    }

    Ok(())
}
