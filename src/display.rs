use crate::CONFIG;
use crate::DISPLAYS;
use winapi::shared::minwindef::BOOL;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::windef::HDC;
use winapi::shared::windef::HMONITOR;
use winapi::shared::windef::LPRECT;
use winapi::shared::windef::RECT;
use winapi::um::winuser::EnumDisplayMonitors;
use winapi::um::winuser::GetSystemMetrics;
use winapi::um::winuser::SM_CMONITORS;

#[derive(Default, Debug, Clone, Copy)]
pub struct Display {
    pub hmonitor: i32,
    pub is_primary: bool,
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

impl Display {
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn new(hmonitor: HMONITOR, rect: RECT) -> Self {
        let mut display = Display::default();
        let config = CONFIG.lock().unwrap();

        display.hmonitor = hmonitor as i32;
        display.left = rect.left;
        display.right = rect.right;
        display.top = rect.top;
        display.bottom = rect.bottom;

        display.is_primary = display.left == 0 && display.top == 0;

        if config.display_app_bar {
            display.bottom -= config.app_bar_height;
        }

        display
    }
}

unsafe extern "system" fn monitor_cb(hmonitor: HMONITOR, _: HDC, rect: LPRECT, _: LPARAM) -> BOOL {
    let display = Display::new(hmonitor, *rect);

    if CONFIG.lock().unwrap().multi_monitor || display.is_primary {
        DISPLAYS.lock().unwrap().push(display);
    }

    1
}

pub fn init() {
    unsafe {
        dbg!(GetSystemMetrics(SM_CMONITORS));
        //is synchronous so don't have to worry about race conditions
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            Some(monitor_cb),
            0,
        );
    }

    println!("{:?}", *DISPLAYS.lock().unwrap());
}

pub fn get_primary_display() -> Display {
    *DISPLAYS
        .lock()
        .unwrap()
        .iter()
        .find(|d| d.is_primary)
        .expect("Couldn't find primary display")
}

pub fn get_display_by_hmonitor(hmonitor: i32) -> Display {
    *DISPLAYS
        .lock()
        .unwrap()
        .iter()
        .find(|d| d.hmonitor == hmonitor)
        .expect(format!("Couldn't find display with hmonitor of {}", hmonitor).as_str())
}

pub fn get_display_by_idx(idx: i32) -> Display {
    let displays = DISPLAYS.lock().unwrap();

    let x: usize = std::cmp::max(displays.len() - (idx as usize), 0);

    *displays
        .get(x)
        .expect(format!("Couldn't get display at index {}", x).as_str())
}
