use crate::CONFIG;
use crate::task_bar;
use crate::app_bar;

use winapi::um::winuser::GetSystemMetrics;
use winapi::um::winuser::SM_CYSCREEN;
use winapi::um::winuser::SM_CXSCREEN;

use log::{debug};

// height of the "display"
// this is not the height of the real display
// it is the real display minus the appbar height
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

        if CONFIG.display_app_bar {
            self.height = self.height - *app_bar::HEIGHT.lock().unwrap();
        }

        if !CONFIG.remove_title_bar {
            self.height = self.height + 9;
            self.width = self.width + 15;
        } 

        // +2 because the taskbar is apparently still on the screen when hidden haha
        let taskbar_is_visible = *task_bar::Y.lock().unwrap() + 2 < self.height;

        if taskbar_is_visible {
            debug!("Taskbar is visible");
        } else {
            debug!("Taskbar is not visible");
        }

        if taskbar_is_visible && !CONFIG.remove_task_bar {
            self.height = self.height - *task_bar::HEIGHT.lock().unwrap();
        }

        debug!("Initialized Display(width: {}, height: {})", self.width, self.height);
    }
}