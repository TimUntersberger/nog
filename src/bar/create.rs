use std::ffi::CString;

use super::{get_windows, redraw::redraw, window_cb, with_bar_by, Bar, BARS};
use crate::{event::Event, message_loop, util, CHANNEL, CONFIG, DISPLAYS};
use log::{debug, error, info};
use winapi::shared::windef::HBRUSH;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::{
    shared::minwindef::HINSTANCE,
    um::winuser::{RegisterClassA, ShowWindow, SW_SHOW, WNDCLASSA},
};

pub fn create() -> Result<(), util::WinApiResultError> {
    info!("Creating appbar");

    let name = "nog_bar";

    let app_bar_bg = CONFIG.lock().bar.color;
    let height = CONFIG.lock().bar.height;

    std::thread::spawn(|| loop {
        std::thread::sleep(std::time::Duration::from_millis(200));

        if get_windows().is_empty() {
            break;
        }

        CHANNEL
            .sender
            .clone()
            .send(Event::RedrawAppBar)
            .expect("Failed to send redraw-app-bar event");
    });

    for display in DISPLAYS.lock().clone() {
        std::thread::spawn(move || unsafe {
            let c_name = CString::new(name).unwrap();

            if with_bar_by(|b| b.display.id == display.id, |b| b.is_some()) {
                error!(
                    "Appbar for monitor {:?} already exists. Aborting",
                    display.id
                );
                return;
            }

            debug!("Creating appbar for display {:?}", display.id);

            let working_area_width = display.working_area_width();

            let instance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());

            let background_brush = CreateSolidBrush(app_bar_bg as u32);

            let class = WNDCLASSA {
                hInstance: instance as HINSTANCE,
                lpszClassName: c_name.as_ptr(),
                lpfnWndProc: Some(window_cb),
                hbrBackground: background_brush as HBRUSH,
                ..WNDCLASSA::default()
            };

            RegisterClassA(&class);

            let window_handle = winapi::um::winuser::CreateWindowExA(
                winapi::um::winuser::WS_EX_NOACTIVATE | winapi::um::winuser::WS_EX_TOPMOST,
                c_name.as_ptr(),
                c_name.as_ptr(),
                winapi::um::winuser::WS_POPUPWINDOW & !winapi::um::winuser::WS_BORDER,
                display.working_area_left(),
                display.working_area_top() - height,
                working_area_width,
                height,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance as HINSTANCE,
                std::ptr::null_mut(),
            );

            let mut bar = Bar::default();

            bar.display = display;
            bar.window = window_handle.into();
            bar.window.show();
            bar.window.redraw();

            BARS.lock().push(bar);

            message_loop::start(|_| true);
        });
    }

    Ok(())
}
