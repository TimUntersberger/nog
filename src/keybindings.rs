use crate::{event::Event, system, system::api, AppState, popup::Popup};
use key::Key;
use keybinding::Keybinding;
use keybinding_type::KeybindingType;
use log::{debug, info};
use modifier::Modifier;
use num_traits::FromPrimitive;
use parking_lot::Mutex;
use std::{
    fmt::Debug,
    sync::atomic::{AtomicBool, Ordering},
    sync::mpsc::channel,
    sync::mpsc::Receiver,
    sync::mpsc::Sender,
    sync::Arc,
    thread,
    time::Duration,
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
    keybindings: Mutex<Vec<Keybinding>>,
    mode: Mutex<Mode>,
}

impl KbManagerInner {
    pub fn new(kbs: Vec<Keybinding>) -> Self {
        Self {
            running: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            mode: Mutex::new(None),
            keybindings: Mutex::new(kbs),
        }
    }

    pub fn unregister_all(&self) {
        self.keybindings.lock().iter().for_each(api::unregister_keybinding);
    }

    pub fn get_keybinding(&self, key: Key, modifier: Modifier) -> Option<Keybinding> {
        let mode = self.mode.lock();
        self.keybindings
            .lock()
            .iter()
            .find(|kb| kb.key == key && kb.modifier == modifier && kb.mode == *mode)
            .map(|kb| kb.clone())
    }

    pub fn get_keybindings_by<T: Fn(&&Keybinding) -> bool>(&self, f: T) -> Vec<Keybinding> {
        self.keybindings.lock().iter().filter(f).map(|kb| kb.clone()).collect()
    }

    pub fn get_keybindings_by_mode(&self, mode: Mode) -> Vec<Keybinding> {
        self.get_keybindings_by(|kb| kb.mode == mode)
    }
}

#[derive(Clone)]
pub struct KbManager {
    inner: Arc<KbManagerInner>,
    sender: Sender<ChanMessage>,
    receiver: Arc<Mutex<Receiver<ChanMessage>>>,
}

impl Debug for KbManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KbManager { }")
    }
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
    pub fn update_bindings(&mut self, kbs: Vec<Keybinding>) {
        *self.inner.keybindings.lock() = kbs;
    }
    fn change_mode(&mut self, mode: Mode) {
        *self.inner.mode.lock() = mode.clone();
        self.sender
            .send(ChanMessage::ChangeMode(mode))
            .expect("Failed to change mode of kb manager");
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
    fn show_keybinding_error(keybinding: &Keybinding, state_arc: Arc<Mutex<AppState>>) {
        let message = format!("Failed to register {:?}.\nAnother running application may already have this binding registered.", &keybinding);
        Popup::error(message, state_arc);
    }
    pub fn start(&self, state_arc: Arc<Mutex<AppState>>) {
        let inner = self.inner.clone();
        let receiver = self.receiver.clone();
        let state = state_arc.clone();

        thread::spawn(move || {
            let receiver = receiver.lock();

            for kb in inner.get_keybindings_by_mode(None) {
                info!("Registering {:?}", kb);
                api::register_keybinding(&kb)
                    .map_err(|_| {
                        let state_arc = state_arc.clone();
                        KbManager::show_keybinding_error(&kb, state_arc); 
                    });
            }

            loop {
                if let Ok(msg) = receiver.try_recv() {
                    debug!("KbManager received {:?}", msg);
                    match msg {
                        ChanMessage::Stop => {
                            debug!("Stopping KbManager");
                            inner.unregister_all();
                            inner.running.store(false, Ordering::SeqCst);
                            break;
                        }
                        ChanMessage::ChangeMode(new_mode) => {
                            // Unregister all keybindings to ensure a clean state
                            inner.unregister_all();

                            // Register all keybinding that belong to the new mode and some
                            // exceptions like ToggleMode keybindings for this mode
                            inner
                                .keybindings
                                .lock()
                                .iter()
                                .filter(|kb| kb.mode == new_mode)
                                .for_each(|kb| { 
                                    api::register_keybinding(kb)
                                        .map_err(|_| {
                                            let state_arc = state_arc.clone();
                                            KbManager::show_keybinding_error(&kb, state_arc); 
                                        });
                                });
                        }
                    };
                }

                if let Some(kb) = do_loop(&inner) {
                    if state.lock().work_mode || kb.typ == KeybindingType::ToggleWorkMode {
                        state
                            .lock()
                            .event_channel
                            .sender
                            .clone()
                            .send(Event::Keybinding(kb))
                            .expect("Failed to send key event");
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        });
    }
    pub fn stop(&mut self) {
        self.inner.clone().stopped.store(true, Ordering::SeqCst);
        self.sender
            .send(ChanMessage::Stop)
            .expect("Failed to stop kb manager");
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn do_loop(inner: &Arc<KbManagerInner>) -> Option<Keybinding> {
    todo!();
}

#[cfg(target_os = "windows")]
fn do_loop(inner: &Arc<KbManagerInner>) -> Option<Keybinding> {
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
