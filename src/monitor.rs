use crate::display::fetch_displays;
use crate::{bar, message_loop, task_bar, CONFIG, DISPLAYS, VISIBLE_WORKSPACES, GRIDS, tile_grid::TileGrid};
use lazy_static::lazy_static;
use std::{
    sync::atomic::{AtomicPtr, Ordering},
    thread,
};
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HWND;
use winapi::shared::windef::HWND__;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::{
    shared::minwindef::HINSTANCE,
    um::winuser::{
        CreateWindowExA, DefWindowProcA, RegisterClassA, UnregisterClassA, WM_DEVICECHANGE,
        WNDCLASSA, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP,
    },
};

unsafe extern "system" fn window_cb(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_DEVICECHANGE && CONFIG.lock().unwrap().multi_monitor {
        let new_displays = fetch_displays();
        let curr_displays = DISPLAYS.lock().unwrap();

        dbg!(new_displays.len());
        dbg!(curr_displays.len());

        let cmp = new_displays.len().cmp(&curr_displays.len());

        let maybe_display = match cmp {
            std::cmp::Ordering::Less => curr_displays
                .iter()
                .find(|x| new_displays.iter().all(|y| x.hmonitor != y.hmonitor)), //remove
            std::cmp::Ordering::Greater => new_displays
                .iter()
                .find(|x| curr_displays.iter().all(|y| x.hmonitor != y.hmonitor)), //add
            std::cmp::Ordering::Equal => None, //nothing
        }
        .cloned();

        if let Some(display) = maybe_display {
            drop(curr_displays);

            bar::close::close();

            let mut vw = VISIBLE_WORKSPACES.lock().unwrap();

            dbg!(&vw);

            match cmp {
                std::cmp::Ordering::Less => {
                    let idx = DISPLAYS
                        .lock()
                        .unwrap()
                        .iter()
                        .enumerate()
                        .find(|(_, d)| d.hmonitor == display.hmonitor)
                        .unwrap()
                        .0;
                    DISPLAYS.lock().unwrap().remove(idx);
                    dbg!(GRIDS.lock().unwrap().iter().filter(|g| g.display.hmonitor == display.hmonitor));
                    vw.remove(&display.hmonitor);
                }
                std::cmp::Ordering::Greater => {
                    DISPLAYS.lock().unwrap().push(display);
                    vw.insert(display.hmonitor, 0);
                }
                std::cmp::Ordering::Equal => {}
            };

            dbg!(&vw);
            drop(vw);

            DISPLAYS.lock().unwrap().sort_by(|x, y| {
                let ordering = y.left.cmp(&x.left);

                if ordering == std::cmp::Ordering::Equal {
                    return y.top.cmp(&x.top);
                }

                ordering
            });

            task_bar::update_task_bars();

            bar::create::create().expect("Failed to create bars");
        }
    }
    DefWindowProcA(hwnd, msg, w_param, l_param)
}

const NAME: &'static str = "nog_monitor_listener";

lazy_static! {
    static ref WINDOW_HANDLE: AtomicPtr<HWND__> = AtomicPtr::new(std::ptr::null_mut());
}

pub fn init() {
    thread::spawn(|| unsafe {
        let instance = GetModuleHandleA(std::ptr::null_mut());
        let class = WNDCLASSA {
            hInstance: instance as HINSTANCE,
            lpszClassName: NAME.as_ptr() as *const i8,
            lpfnWndProc: Some(window_cb),
            ..WNDCLASSA::default()
        };

        RegisterClassA(&class);

        let window_handle = CreateWindowExA(
            WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP,
            NAME.as_ptr() as *const i8,
            NAME.as_ptr() as *const i8,
            0,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance as HINSTANCE,
            std::ptr::null_mut(),
        );

        WINDOW_HANDLE.store(window_handle, Ordering::SeqCst);

        message_loop::start_limited(WM_DEVICECHANGE, WM_DEVICECHANGE, |_| true);
    });
}

pub fn cleanup() {
    unsafe {
        UnregisterClassA(
            NAME.as_ptr() as *const i8,
            GetModuleHandleA(std::ptr::null_mut()),
        );
    }
}
