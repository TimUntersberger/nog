use crate::util;
use crate::Event;
use crate::CHANNEL;
use lazy_static::lazy_static;
use log::debug;
use std::sync::Mutex;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::LONG;
use winapi::shared::windef::HWINEVENTHOOK;
use winapi::shared::windef::HWND;
use winapi::um::winuser::DispatchMessageW;
use winapi::um::winuser::PeekMessageW;
use winapi::um::winuser::SetWinEventHook;
use winapi::um::winuser::TranslateMessage;
use winapi::um::winuser::EVENT_MAX;
use winapi::um::winuser::EVENT_MIN;
use winapi::um::winuser::EVENT_OBJECT_DESTROY;
use winapi::um::winuser::EVENT_OBJECT_HIDE;
use winapi::um::winuser::EVENT_OBJECT_SHOW;
use winapi::um::winuser::EVENT_SYSTEM_FOREGROUND;
use winapi::um::winuser::MSG;
use winapi::um::winuser::OBJID_WINDOW;
use winapi::um::winuser::PM_REMOVE;

static mut HOOK: Option<HWINEVENTHOOK> = None;

lazy_static! {
    static ref UNREGISTER: Mutex<bool> = Mutex::new(false);
}

#[derive(Clone, Copy, Debug)]
pub enum WinEventType {
    Destroy,
    Hide,
    Show(bool),
    FocusChange,
}

impl WinEventType {
    fn from_u32(v: u32) -> Option<Self> {
        if v == EVENT_OBJECT_DESTROY {
            Some(Self::Destroy)
        } else if v == EVENT_OBJECT_SHOW {
            Some(Self::Show(false))
        } else if v == EVENT_SYSTEM_FOREGROUND {
            Some(Self::FocusChange)
        } else if v == EVENT_OBJECT_HIDE {
            Some(Self::Hide)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WinEvent {
    pub typ: WinEventType,
    pub hwnd: i32,
}

unsafe extern "system" fn handler(
    _: HWINEVENTHOOK,
    event_code: DWORD,
    window_handle: HWND,
    object_type: LONG,
    _: LONG,
    _: DWORD,
    _: DWORD,
) {
    if object_type != OBJID_WINDOW {
        return;
    }

    let win_event_type = match WinEventType::from_u32(event_code) {
        Some(event) => event,
        None => return,
    };
    let event = Event::WinEvent(WinEvent {
        typ: win_event_type,
        hwnd: window_handle as i32,
    });

    CHANNEL.sender.clone().send(event).unwrap();
}

pub fn register() -> Result<(), util::WinApiResultError> {
    std::thread::spawn(|| unsafe {
        let mut msg: MSG = MSG::default();

        debug!("Registering win event hook");

        let hook = util::winapi_ptr_to_result(SetWinEventHook(
            EVENT_MIN,
            EVENT_MAX,
            std::ptr::null_mut(),
            Some(handler),
            0,
            0,
            0,
        ))
        .unwrap();

        HOOK = Some(hook);

        loop {
            while PeekMessageW(&mut msg, 0 as HWND, 0, 0, PM_REMOVE) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            if *UNREGISTER.lock().unwrap() {
                debug!("Win event hook unregistered");
                *UNREGISTER.lock().unwrap() = false;
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    });

    Ok(())
}

pub fn unregister() -> Result<(), util::WinApiResultError> {
    debug!("Unregistering win event hook");

    *UNREGISTER.lock().unwrap() = true;

    Ok(())
}
