use crate::{event::Event, system, system::api, AppState};
use key::Key;
use keybinding::Keybinding;
use keybinding_type::KeybindingType;
use log::{debug, info};
use modifier::Modifier;
use num_traits::FromPrimitive;
use parking_lot::Mutex;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::mpsc::channel,
    sync::mpsc::Receiver,
    sync::mpsc::Sender,
    sync::Arc,
    thread,
};

pub mod key;
pub mod keybinding;
pub mod keybinding_type;
pub mod modifier;

pub type Mode = Option<String>;

#[derive(Debug, Clone)]
enum ChanMessage {
    Stop,
    ChangeMode(Mode),
}

struct KbManagerInner {
    running: AtomicBool,
    stopped: AtomicBool,
    keybindings: Vec<Keybinding>,
    mode: Mutex<Mode>,
}

impl KbManagerInner {
    pub fn new(kbs: Vec<Keybinding>) -> Self {
        Self {
            running: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            mode: Mutex::new(None),
            keybindings: kbs,
        }
    }

    pub fn get_keybinding(&self, key: Key, modifier: Modifier) -> Option<&Keybinding> {
        self.keybindings
            .iter()
            .find(|kb| kb.key == key && kb.modifier == modifier)
    }

    pub fn get_keybindings_by<T: Fn(&&Keybinding) -> bool>(&self, f: T) -> Vec<&Keybinding> {
        self.keybindings.iter().filter(f).collect()
    }

    pub fn get_keybindings_by_mode(&self, mode: Mode) -> Vec<&Keybinding> {
        self.get_keybindings_by(|kb| kb.mode == mode)
    }
}

pub struct KbManager {
    inner: Arc<KbManagerInner>,
    sender: Sender<ChanMessage>,
    receiver: Arc<Mutex<Receiver<ChanMessage>>>,
}

impl KbManager {
    pub fn new(kbs: Vec<Keybinding>) -> Self {
        let (sender, receiver) = channel();
        Self {
            inner: Arc::new(KbManagerInner::new(kbs)),
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
    fn change_mode(&mut self, mode: Mode) {
        *self.inner.mode.lock() = mode.clone();
        self.sender.send(ChanMessage::ChangeMode(mode));
    }
    pub fn enter_mode(&mut self, mode: &str) {
        self.change_mode(Some(mode.into()));
    }
    pub fn leave_mode(&mut self) {
        self.change_mode(None);
    }
    pub fn get_mode(&self) -> Mode {
        self.inner.clone().mode.lock().clone()
    }
    pub fn start(&self, state_arc: Arc<Mutex<AppState>>) {
        let inner = self.inner.clone();
        let receiver = self.receiver.clone();
        let state = state_arc.clone();

        thread::spawn(move || {
            let receiver = receiver.lock();

            for kb in inner.get_keybindings_by_mode(None) {
                info!("Registering {:?}", kb);
                api::register_keybinding(&kb);
            }

            loop {
                if let Ok(msg) = receiver.try_recv() {
                    debug!("KbManager received {:?}", msg);
                    let keep_running = match msg {
                        ChanMessage::Stop => false,
                        ChanMessage::ChangeMode(new_mode) => {
                            // Unregister all keybindings to ensure a clean state
                            inner
                                .keybindings
                                .iter()
                                .for_each(api::unregister_keybinding);

                            // Register all keybinding that belong to the new mode and some
                            // exceptions like ToggleMode keybindings for this mode
                            inner
                                .keybindings
                                .iter()
                                .filter(|kb| {
                                    kb.mode == new_mode
                                        || kb.typ
                                            == KeybindingType::ToggleMode(
                                                new_mode.clone().unwrap_or_default(),
                                            )
                                })
                                .for_each(api::register_keybinding);

                            true
                        }
                    };

                    if !keep_running {
                        debug!("Stopping KbManager");
                        inner.running.store(false, Ordering::SeqCst);
                        //TODO: Unregister all keybindings except toggle work mode
                        break;
                    }
                }

                if let Some(kb) = do_loop(&inner) {
                    if state.lock().work_mode || kb.typ == KeybindingType::ToggleWorkMode {
                        state
                            .lock()
                            .event_channel
                            .sender
                            .clone()
                            .send(Event::Keybinding(kb.clone()))
                            .expect("Failed to send key event");
                    }
                }
            }
        });
    }
    pub fn stop(&mut self) {
        self.inner.clone().stopped.store(true, Ordering::SeqCst);
        self.sender.send(ChanMessage::Stop);
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn do_loop(inner: &Arc<KbManagerInner>) -> Option<&Keybinding> {
    todo!();
}

#[cfg(target_os = "windows")]
fn do_loop(inner: &Arc<KbManagerInner>) -> Option<&Keybinding> {
    use winapi::um::winuser::WM_HOTKEY;

    if let Some(msg) = system::win::api::get_current_window_msg() {
        if msg.message != WM_HOTKEY {
            return None;
        }

        let modifier = Modifier::from_bits((msg.lParam & 0xffff) as u32).unwrap();

        if let Some(key) = Key::from_isize(msg.lParam >> 16) {
            return inner.get_keybinding(key, modifier);
        }
    }

    None
}
