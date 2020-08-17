use super::{get_windows, FONT};
use log::{debug, info};
use std::ffi::CString;
use winapi::um::winuser::UnregisterClassA;

pub fn close() {
    unsafe {
        info!("Closing appbar");

        for window in get_windows() {
            window.close();
        }

        let name = CString::new("nog_bar").expect("Failed to transform string to cstring");

        debug!("Unregistering window class");

        UnregisterClassA(
            name.as_ptr(),
            winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut()),
        );

        *FONT.lock().unwrap() = 0;
    }
}
