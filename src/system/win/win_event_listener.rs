use crate::{
    event::Event, message_loop, win_event_handler::win_event::WinEvent,
    win_event_handler::win_event_type::WinEventType,
window::WindowMsg, AppState, event::EventChannel};
use parking_lot::Mutex;
use log::debug;
use std::{
    ptr, sync::atomic::AtomicBool, sync::atomic::AtomicPtr, sync::atomic::Ordering, sync::Arc,
    thread, time::Duration,
sync::mpsc::Sender, sync::mpsc::Receiver, sync::mpsc::channel};
use winapi::{
    shared::{minwindef::*, ntdef::*, windef::*},
    um::winuser::*,
};
use lazy_static::lazy_static;
use super::nullable_to_result;

const WM_IDENT: u32 = WM_APP + 30;

lazy_static! {
    static ref CHAN: Arc<Mutex<(Sender<Event>, Receiver<Event>)>> = Arc::new(Mutex::new(channel()));
}

unsafe extern "system" fn handler(
    _: HWINEVENTHOOK,
    event_code: DWORD,
    hwnd: HWND,
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
        window: hwnd.into(),
    });

    CHAN.lock().0.send(event);
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
    pub fn start(&self, channel: &EventChannel) {
        let hook = self.hook.clone();
        let stopped = self.stopped.clone();
        let sender = channel.sender.clone();

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
                if stopped.load(Ordering::SeqCst) {
                    debug!("Win event hook unregistered");
                    stopped.store(false, Ordering::SeqCst);
                    return false;
                }

                if let Ok(event) = CHAN.lock().1.try_recv() {
                    sender.send(event);
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
