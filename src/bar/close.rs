use super::{get_windows, FONT};
use log::{debug, info};
use std::ffi::CString;
use winapi::shared::windef::HWND;
use winapi::um::shellapi::{SHAppBarMessage, ABM_REMOVE, APPBARDATA};
use winapi::um::winuser::UnregisterClassA;

pub fn close() {
    unsafe {
        info!("Closing appbar");

        for window in get_windows() {
            let mut appbar_data: APPBARDATA = APPBARDATA {
                cbSize: 4 + 4 + 4 + 4 + 16 + 4,
                hWnd: window.id as HWND,
                ..Default::default()
            };

            SHAppBarMessage(ABM_REMOVE, &mut appbar_data as *mut APPBARDATA);

            window.close();
        }

        let name = CString::new("nog_bar").expect("Failed to transform string to cstring");

        debug!("Unregistering window class");

        UnregisterClassA(
            name.as_ptr(),
            winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut()),
        );

        *FONT.lock() = 0;
    }
}
