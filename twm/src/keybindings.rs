use crate::{event::Event, popup::Popup, system, system::api, AppState};
use key::Key;
use keybinding::Keybinding;
use log::{debug, info};
use modifier::Modifier;
use num_traits::FromPrimitive;
use parking_lot::Mutex;
use std::collections::HashMap;
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
pub mod modifier;

pub type Mode = Option<String>;

#[derive(Debug, Clone)]
pub enum ChanMessage {
    Stop,
    ChangeMode(Mode),
    ModeCbExecuted,
}

struct KbManagerInner {
    running: AtomicBool,
    stopped: AtomicBool,
    /// Holds all of the handlers that get called when entering a mode
    /// Key is mode name and value is the callback id
    mode_handlers: HashMap<String, usize>,
    keybindings: Vec<Keybinding>,
    mode_keybindings: Mutex<HashMap<String, Vec<Keybinding>>>,
    mode: Mutex<Mode>,
}

impl KbManagerInner {
    pub fn new(kbs: Vec<Keybinding>, handlers: HashMap<String, usize>) -> Self {
        Self {
            running: AtomicBool::new(false),
            mode_handlers: handlers,
            stopped: AtomicBool::new(false),
            mode: Mutex::new(None),
            keybindings: kbs,
            mode_keybindings: Mutex::new(HashMap::new()),
        }
    }

    pub fn unregister_all(&self) {
        self.keybindings.iter().for_each(api::unregister_keybinding);
    }

    pub fn register_all(&self, state_arc: Arc<Mutex<AppState>>) {
        let state = state_arc.clone();
        self.keybindings.iter().for_each(|kb| {
            api::register_keybinding(kb).map_err(|_| {
                KbManager::show_keybinding_error(&kb, state.clone());
            });
        });
    }

    pub fn get_keybinding(&self, key: Key, modifier: Modifier) -> Option<Keybinding> {
        let mode = self.mode.lock();
        match mode.as_ref() {
            Some(mode) => self
                .mode_keybindings
                .lock()
                .get(mode)
                .unwrap()
                .iter()
                .find(|kb| kb.key == key && kb.modifier == modifier)
                .map(|kb| kb.clone()),
            None => self
                .keybindings
                .iter()
                .find(|kb| kb.key == key && kb.modifier == modifier)
                .map(|kb| kb.clone()),
        }
    }
}

#[derive(Clone)]
pub struct KbManager {
    inner: Arc<KbManagerInner>,
    pub sender: Sender<ChanMessage>,
    receiver: Arc<Mutex<Receiver<ChanMessage>>>,
}

impl Debug for KbManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KbManager { }")
    }
}

impl KbManager {
    pub fn new(kbs: Vec<Keybinding>, handlers: HashMap<String, usize>) -> Self {
        let (sender, receiver) = channel();
        Self {
            inner: Arc::new(KbManagerInner::new(kbs, handlers)),
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
    fn change_mode(&mut self, mode: Mode) {
        self.sender
            .send(ChanMessage::ChangeMode(mode.clone()))
            .expect("Failed to change mode of kb manager");
    }
    pub fn enter_mode(&mut self, mode: &str) {
        self.change_mode(Some(mode.into()));
    }
    pub fn leave_mode(&mut self) {
        self.change_mode(None);
    }
    pub fn add_mode_keybinding(&mut self, kb: Keybinding) {
        if let Some(mode) = self.get_mode() {
            self.inner
                .mode_keybindings
                .lock()
                .get_mut(&mode)
                .unwrap()
                .push(kb);
        }
    }
    pub fn get_mode(&self) -> Mode {
        self.inner.mode.lock().clone()
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

            for kb in &inner.keybindings {
                info!("Registering {:?}", kb);
                api::register_keybinding(&kb).map_err(|_| {
                    let state_arc = state_arc.clone();
                    KbManager::show_keybinding_error(&kb, state_arc);
                });
            }

            loop {
                if let Ok(msg) = receiver.try_recv() {
                    debug!("KbManager received {:?}", msg);
                    match msg {
                        ChanMessage::ModeCbExecuted => unreachable!(),
                        ChanMessage::Stop => {
                            debug!("Stopping KbManager");
                            inner.unregister_all();
                            inner.running.store(false, Ordering::SeqCst);
                            break;
                        }
                        ChanMessage::ChangeMode(new_mode) => {
                            // Unregister all keybindings to ensure a clean state
                            inner.unregister_all();

                            if let Some(mode) = new_mode.as_ref() {
                                *inner.mode.lock() = new_mode.clone();
                                if !inner.mode_keybindings.lock().contains_key(mode) {
                                    if let Some(id) = inner.mode_handlers.get(mode) {
                                        let sender = state.lock().event_channel.sender.clone();
                                        inner
                                            .mode_keybindings
                                            .lock()
                                            .insert(mode.clone(), Vec::new());

                                        sender
                                            .send(Event::CallCallback {
                                                idx: *id,
                                                is_mode_callback: true,
                                            })
                                            .unwrap();

                                        receiver.recv().unwrap();
                                    }
                                }

                                let kbs_lock = inner.mode_keybindings.lock();
                                let kbs = kbs_lock.get(mode).unwrap();

                                for kb in kbs.iter() {
                                    api::register_keybinding(kb).map_err(|_| {
                                        let state_arc = state_arc.clone();
                                        KbManager::show_keybinding_error(&kb, state_arc);
                                    });
                                }
                            } else {
                                let mut mode_lock = inner.mode.lock();
                                let kbs_lock = inner.mode_keybindings.lock();
                                let kbs = kbs_lock.get(mode_lock.as_ref().unwrap()).unwrap();

                                for kb in kbs.iter() {
                                    api::unregister_keybinding(kb);
                                }

                                *mode_lock = new_mode.clone();

                                inner.register_all(state_arc.clone());
                            }
                        }
                    };
                }

                if let Some(kb) = do_loop(&inner) {
                    let work_mode = state.lock().work_mode;
                    if work_mode || kb.always_active {
                        state
                            .lock()
                            .event_channel
                            .sender
                            .clone()
                            .send(Event::Keybinding(kb))
                            .expect("Failed to send key event");
                    }
                }

                thread::sleep(Duration::from_millis(100));
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
