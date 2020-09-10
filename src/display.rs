use crate::task_bar;
use crate::CONFIG;
use crate::DISPLAYS;
use std::cmp::Ordering;
use winapi::shared::minwindef::BOOL;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::windef::{HDC, HMONITOR, LPRECT, RECT};
use winapi::um::shellscalingapi::{GetDpiForMonitor, MDT_RAW_DPI};
use winapi::um::winuser::EnumDisplayMonitors;

#[derive(Default, Debug, Clone, Copy)]
pub struct Display {
    pub hmonitor: i32,
    pub dpi: u32,
    pub is_primary: bool,
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
    pub task_bar: Option<task_bar::TaskBar>,
}

impl Display {
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn working_area_height(&self) -> i32 {
        let task_bar_height = if let Some(task_bar) = self.task_bar {
            match task_bar.position {
                task_bar::TaskBarPosition::Top | task_bar::TaskBarPosition::Bottom => {
                    task_bar.height
                }
                _ => 0,
            }
        } else {
            0
        };

        self.height() - task_bar_height
    }
    pub fn working_area_width(&self) -> i32 {
        let task_bar_width = if self.task_bar.is_some() {
            let task_bar = self.task_bar.unwrap();
            match task_bar.position {
                task_bar::TaskBarPosition::Left | task_bar::TaskBarPosition::Right => {
                    task_bar.width
                }
                _ => 0,
            }
        } else {
            0
        };

        self.width() - task_bar_width
    }
    pub fn working_area_top(&self) -> i32 {
        let offset = if let Some(task_bar) = self.task_bar {
            match task_bar.position {
                task_bar::TaskBarPosition::Top => task_bar.height,
                _ => 0,
            }
        } else {
            0
        };
        self.top + offset
    }
    pub fn working_area_left(&self) -> i32 {
        let offset = if let Some(task_bar) = self.task_bar {
            match task_bar.position {
                task_bar::TaskBarPosition::Left => task_bar.width,
                _ => 0,
            }
        } else {
            0
        };
        self.left + offset
    }
    pub fn new(hmonitor: HMONITOR, rect: RECT) -> Self {
        let mut display = Display::default();
        let config = CONFIG.lock();
        let mut dpi_x: u32 = 0;
        let mut dpi_y: u32 = 0;

        unsafe {
            GetDpiForMonitor(hmonitor, MDT_RAW_DPI, &mut dpi_x, &mut dpi_y);
        }
        display.dpi = dpi_x;
        display.hmonitor = hmonitor as i32;
        display.left = rect.left;
        display.right = rect.right;
        display.top = rect.top;
        display.bottom = rect.bottom;

        display.is_primary = display.left == 0 && display.top == 0;

        if config.display_app_bar {
            display.bottom -= config.bar.height;
        }

        display
    }
}

unsafe extern "system" fn monitor_cb(hmonitor: HMONITOR, _: HDC, rect: LPRECT, _: LPARAM) -> BOOL {
    let display = Display::new(hmonitor, *rect);

    if CONFIG.lock().multi_monitor || display.is_primary {
        DISPLAYS.lock().push(display);
    }

    1
}

pub fn init() {
    unsafe {
        //is synchronous so don't have to worry about race conditions
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            Some(monitor_cb),
            0,
        );
    }

    {
        let mut displays = DISPLAYS.lock();

        displays.sort_by(|x, y| {
            let ordering = y.left.cmp(&x.left);

            if ordering == Ordering::Equal {
                return y.top.cmp(&x.top);
            }

            ordering
        });
    }

    task_bar::update_task_bars();
}

pub fn get_primary_display() -> Display {
    *DISPLAYS
        .lock()
        .iter()
        .find(|d| d.is_primary)
        .expect("Couldn't find primary display")
}

pub fn get_display_by_hmonitor(hmonitor: i32) -> Display {
    *DISPLAYS
        .lock()
        .iter()
        .find(|d| d.hmonitor == hmonitor)
        .expect(format!("Couldn't find display with hmonitor of {}", hmonitor).as_str())
}

pub fn get_display_by_idx(idx: i32) -> Display {
    let displays = DISPLAYS.lock();

    let x: usize = if idx == -1 {
        0
    } else {
        std::cmp::max(displays.len() - (idx as usize), 0)
    };

    *displays
        .get(x)
        .expect(format!("Couldn't get display at index {}", x).as_str())
}
