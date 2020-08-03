use winapi::um::winuser::{PeekMessageW, MSG, TranslateMessage, DispatchMessageW, PM_REMOVE, WM_QUIT};

pub fn start(cb: impl Fn(Option<MSG>) -> bool) {
    let mut msg: MSG = MSG::default();
    loop {
        let mut value: Option<MSG> = None;
        unsafe {
            if PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);

                if msg.message == WM_QUIT {
                    break;
                }

                value = Some(msg);
            }
        }
        
        if !cb(value) {
            break;
        }
    }
}
