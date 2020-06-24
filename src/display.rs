use crate::CONFIG;
use crate::task_bar;
use crate::app_bar;

use winapi::um::winuser::GetSystemMetrics;
use winapi::um::winuser::SM_CYSCREEN;
use winapi::um::winuser::SM_CXSCREEN;

use log::{debug};

#[derive(Default)]
pub struct Display {
    pub height: i32,
    pub width: i32
}

impl Display {
    pub fn init(&mut self) {
        unsafe {
            self.height = GetSystemMetrics(SM_CYSCREEN);
            self.width = GetSystemMetrics(SM_CXSCREEN);
        }

        // +2 because the taskbar is apparently still on the screen when hidden haha
        let taskbar_is_visible = *task_bar::Y.lock().unwrap() + 2 < self.height;

        if taskbar_is_visible {
            debug!("Taskbar is visible");
        } else {
            debug!("Taskbar is not visible");
        }

        if taskbar_is_visible && !CONFIG.remove_task_bar {
            self.height -= *task_bar::HEIGHT.lock().unwrap();
        }

        debug!("Initialized Display(width: {}, height: {})", self.width, self.height);
    }
}