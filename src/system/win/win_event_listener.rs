use crate::{
    bar::get_bar_by_win_id, event::Event, message_loop, win_event_handler::win_event::WinEvent,
    win_event_handler::win_event_type::WinEventType, CHANNEL,
};
use log::debug;
use std::{
    ptr, sync::atomic::AtomicBool, sync::atomic::AtomicPtr, sync::atomic::Ordering, sync::Arc,
    thread, time::Duration,
};
use winapi::{
    shared::{minwindef::*, ntdef::*, windef::*},
    um::winuser::*,
};

use super::nullable_to_result;

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

    if get_bar_by_win_id(window_handle.into()).is_some() {
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

#[derive(Debug)]
pub struct WinEventListener {
    stopped: Arc<AtomicBool>,
    hook: Arc<AtomicPtr<HWINEVENTHOOK__>>,
}

impl Default for WinEventListener {
    fn default() -> Self {
        Self {
            stopped: Arc::new(AtomicBool::new(false)),
            hook: Arc::new(AtomicPtr::new(ptr::null_mut())),
        }
    }
}

impl WinEventListener {
    pub fn start(&self) {
        let hook = self.hook.clone();
        let stopped = self.stopped.clone();

        thread::spawn(move || unsafe {
            debug!("Registering win event hook");

            let hook_ptr = nullable_to_result(SetWinEventHook(
                EVENT_MIN,
                EVENT_MAX,
                std::ptr::null_mut(),
                Some(handler),
                0,
                0,
                0,
            ) as i32)
            .unwrap();

            hook.store(hook_ptr as HWINEVENTHOOK, Ordering::SeqCst);

            message_loop::start(|msg| {
                if let Some(msg) = msg {
                    dbg!(msg.lParam);
                }
                if stopped.load(Ordering::SeqCst) {
                    debug!("Win event hook unregistered");
                    stopped.store(false, Ordering::SeqCst);
                    return false;
                }

                thread::sleep(Duration::from_millis(5));

                true
            });
        });
    }

    pub fn stop(&self) {
        debug!("Unregistering win event hook");

        self.stopped.clone().store(true, Ordering::SeqCst);
    }
}
