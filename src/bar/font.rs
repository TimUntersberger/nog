use super::FONT;
use crate::CONFIG;
use log::debug;
use std::ffi::CString;
use winapi::shared::windef::HDC;
use winapi::um::wingdi::CreateFontIndirectA;
use winapi::um::wingdi::SelectObject;
use winapi::um::wingdi::LOGFONTA;

pub fn set_font(dc: HDC) {
    unsafe {
        SelectObject(dc, *FONT.lock() as *mut std::ffi::c_void);
    }
}

pub fn load_font() {
    if *FONT.lock() != 0 {
        return;
    }
    unsafe {
        let mut logfont = LOGFONTA::default();
        let mut font_name: [i8; 32] = [0; 32];
        let app_bar_font = CONFIG.lock().bar.font.clone();
        let app_bar_font_size = CONFIG.lock().bar.font_size;

        for (i, byte) in CString::new(app_bar_font.as_str())
            .unwrap()
            .as_bytes()
            .iter()
            .enumerate()
        {
            font_name[i] = *byte as i8;
        }

        logfont.lfHeight = app_bar_font_size;
        logfont.lfFaceName = font_name;

        let font = CreateFontIndirectA(&logfont) as i32;

        debug!("Using font {}", font);

        *FONT.lock() = font;
    }
}
