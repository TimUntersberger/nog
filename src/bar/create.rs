use super::{
    window_cb,
    WINDOWS, redraw::redraw,
};
use crate::{event::Event, message_loop, task_bar::HEIGHT, util, CHANNEL, CONFIG, DISPLAYS};
use log::{debug, error, info};
use winapi::shared::windef::HBRUSH;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::{
    shared::minwindef::HINSTANCE,
    um::winuser::{RegisterClassA, ShowWindow, SW_SHOW, WNDCLASSA},
};

pub fn create() -> Result<(), util::WinApiResultError> {
    info!("Creating appbar");

    let name = "wwm_app_bar";

    let mut height_guard = HEIGHT.lock().unwrap();

    let app_bar_bg = CONFIG.lock().unwrap().app_bar_bg;

    *height_guard = CONFIG.lock().unwrap().app_bar_height;

    let height = *height_guard;

    std::thread::spawn(|| loop {
        std::thread::sleep(std::time::Duration::from_millis(950));
        if WINDOWS.lock().unwrap().is_empty() {
            break;
        }

        CHANNEL
            .sender
            .clone()
            .send(Event::RedrawAppBar)
            .expect("Failed to send redraw-app-bar event");
    });

    for display in DISPLAYS.lock().unwrap().clone() {
        std::thread::spawn(move || unsafe {
            if WINDOWS
                .lock()
                .unwrap()
                .contains_key(&(display.hmonitor as i32))
            {
                error!(
                    "Appbar for monitor {} already exists. Aborting",
                    display.hmonitor as i32
                );
            }

            debug!("Creating appbar for display {}", display.hmonitor as i32);

            let display_width = display.width();
            //TODO: Handle error
            let instance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());
            //TODO: Handle error
            let background_brush = CreateSolidBrush(app_bar_bg as u32);

            let class = WNDCLASSA {
                hInstance: instance as HINSTANCE,
                lpszClassName: name.as_ptr() as *const i8,
                lpfnWndProc: Some(window_cb),
                hbrBackground: background_brush as HBRUSH,
                ..WNDCLASSA::default()
            };

            RegisterClassA(&class);

            //TODO: handle error
            let window_handle = winapi::um::winuser::CreateWindowExA(
                winapi::um::winuser::WS_EX_NOACTIVATE | winapi::um::winuser::WS_EX_TOPMOST,
                name.as_ptr() as *const i8,
                name.as_ptr() as *const i8,
                winapi::um::winuser::WS_POPUPWINDOW & !winapi::um::winuser::WS_BORDER,
                display.left,
                display.top,
                display_width,
                height,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance as HINSTANCE,
                std::ptr::null_mut(),
            );

            WINDOWS
                .lock()
                .unwrap()
                .insert(display.hmonitor as i32, window_handle as i32);

            ShowWindow(window_handle, SW_SHOW);
            redraw();

            message_loop::start(|_| true);
        });
    }

    Ok(())
}
