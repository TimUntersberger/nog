use std::{thread, time::Duration};
use winapi::um::winuser::{
    DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_QUIT,
};

pub fn start(cb: impl Fn(Option<MSG>) -> bool) {
    start_with_sleep(10, cb);
}

pub fn start_with_sleep(sleep: u64, cb: impl Fn(Option<MSG>) -> bool) {
    let mut msg: MSG = MSG::default();
    loop {
        let mut value: Option<MSG> = None;
        unsafe {
            if PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);

                value = Some(msg);
            }
        }

        thread::sleep(Duration::from_millis(sleep));

        if !cb(value) {
            break;
        }
    }
}
