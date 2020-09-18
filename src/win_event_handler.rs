use crate::bar;
use crate::util;
use crate::Event;
use crate::{message_loop, CHANNEL};
use lazy_static::lazy_static;
use log::debug;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use win_event::WinEvent;
use win_event_type::WinEventType;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::LONG;
use winapi::shared::windef::HWND;
use winapi::shared::windef::{HWINEVENTHOOK, HWINEVENTHOOK__};
use winapi::um::winuser::SetWinEventHook;
use winapi::um::winuser::EVENT_MAX;
use winapi::um::winuser::EVENT_MIN;
use winapi::um::winuser::OBJID_WINDOW;

pub mod win_event;
pub mod win_event_code;
pub mod win_event_type;

lazy_static! {
    static ref HOOK: AtomicPtr<HWINEVENTHOOK__> = AtomicPtr::new(std::ptr::null_mut());
    static ref UNREGISTER: AtomicBool = AtomicBool::new(false);
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

    if bar::get_bar_by_hwnd(window_handle.into()).is_some() {
        return;
    }

    let win_event_type = match WinEventType::from_u32(event_code) {
        Some(event) => event,
        None => return,
    };

    let event = Event::WinEvent(WinEvent {
        typ: win_event_type,
        window: window_handle.into(),
    });

    CHANNEL.sender.clone().send(event).unwrap();
}

pub fn register() -> Result<(), util::WinApiResultError> {
    std::thread::spawn(|| unsafe {
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

        HOOK.store(hook, Ordering::SeqCst);

        message_loop::start(|_| {
            if UNREGISTER.load(Ordering::SeqCst) {
                debug!("Win event hook unregistered");
                UNREGISTER.store(false, Ordering::SeqCst);
                return false;
            }

            std::thread::sleep(std::time::Duration::from_millis(5));

            true
        });
    });

    Ok(())
}

pub fn unregister() -> Result<(), util::WinApiResultError> {
    debug!("Unregistering win event hook");

    UNREGISTER.store(true, Ordering::SeqCst);

    Ok(())
}
