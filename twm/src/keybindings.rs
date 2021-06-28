use crate::{keyboardhook::{self, InputEvent}, config::Config, event::Event, popup::Popup, system, system::api, AppState};
use key::Key;
use keybinding::Keybinding;
use log::{debug, error, info};
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
pub mod modifier;

pub type Mode = Option<String>;

#[derive(Debug, Clone)]
pub enum ChanMessage {
    Stop,
    LeaveWorkMode,
    EnterWorkMode,
    RegisterKeybindings,
    RegisterKeybinding(Keybinding),
    RegisterKeybindingBatch(Vec<Keybinding>),
    UnregisterKeybinding(Keybinding),
    UnregisterKeybindingBatch(Vec<Keybinding>),
    UnregisterKeybindings,
    ModeCbExecuted,
}

struct KbManagerInner {
    running: AtomicBool,
    stopped: AtomicBool,
    allow_right_alt: bool,
    state: Arc<Mutex<AppState>>
}

impl KbManagerInner {
    pub fn new(state: Arc<Mutex<AppState>>, allow_right_alt: bool) -> Self {
        Self {
            running: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            state,
            allow_right_alt,
        }
    }

    pub fn unregister_kb(&self, kb: &Keybinding) {
        info!("Unregistering {:?}", kb);
        api::unregister_keybinding(kb).map_err(|err| {
            error!("WINAPI {:?}", err);
        });
    }

    pub fn unregister_all(&self) {
        self.state
            .lock()
            .config
            .keybindings
            .iter()
            .for_each(|kb| self.unregister_kb(kb));
    }

    pub fn register_kb(&self, kb: &Keybinding) -> Result<(), String> {
        info!("Registering {:?}", kb);
        api::register_keybinding(kb).map_err(|err| {
            let msg = KbManager::make_keybinding_error(&kb);
            error!("{}", &msg);
            msg
        })
    }

    pub fn register_all(&self, kbs: &Vec<&Keybinding>, state_arc: Arc<Mutex<AppState>>) {
        let mut errors = Vec::new();

        for kb in kbs {
            if let Err(msg) = self.register_kb(kb) {
                errors.push(msg);
            }
        }

        if !errors.is_empty() {
            Popup::error(errors, state_arc.clone());
        }
    }

    pub fn get_keybinding(&self, key: Key, modifier: Modifier) -> Option<Keybinding> {
                self.state
                .lock()
                .config
                .keybindings
                .iter()
                .find(|kb| kb.key == key && kb.modifier == modifier)
                .map(|kb| kb.clone())
    }
}

#[derive(Clone)]
pub struct KbManager {

    inner: Arc<Mutex<KbManagerInner>>,
    pub sender: Sender<ChanMessage>,
    receiver: Arc<Mutex<Receiver<ChanMessage>>>,
}

impl Debug for KbManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KbManager { }")
    }
}

impl KbManager {
    pub fn new(state: Arc<Mutex<AppState>>, allow_right_alt: bool) -> Self {
        let (sender, receiver) = channel();
        Self {
            inner: Arc::new(Mutex::new(KbManagerInner::new(state, allow_right_alt))),
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
    pub fn set_state(&mut self, state: Arc<Mutex<AppState>>) {
        self.inner.lock().state = state;
    }
    pub fn update_configuration(&self, config: &Config) {
        self.inner.lock().allow_right_alt = config.allow_right_alt;
    }
    pub fn leave_work_mode(&self) {
        self.sender
            .send(ChanMessage::LeaveWorkMode)
            .expect("Failed to send leave work mode");
    }
    pub fn enter_work_mode(&self) {
        self.sender
            .send(ChanMessage::EnterWorkMode)
            .expect("Failed to send enter work mode");
    }
    pub fn unregister_keybindings(&self) {
        self.sender
            .send(ChanMessage::UnregisterKeybindings)
            .expect("Failed to send UnregisterKeybindings");
    }
    pub fn unregister_keybinding(&self, kb: Keybinding) {
        self.sender
            .send(ChanMessage::UnregisterKeybinding(kb))
            .expect("Failed to send UnregisterKeybindings");
    }
    pub fn unregister_keybinding_batch(&self, kbs: Vec<Keybinding>) {
        self.sender
            .send(ChanMessage::UnregisterKeybindingBatch(kbs))
            .expect("Failed to send UnregisterKeybindingBatch");
    }
    pub fn register_keybindings(&self) {
        self.sender
            .send(ChanMessage::RegisterKeybindings)
            .expect("Failed to send RegisterKeybindings");
    }
    pub fn register_keybinding(&self, kb: Keybinding) {
        self.sender
            .send(ChanMessage::RegisterKeybinding(kb))
            .expect("Failed to send RegisterKeybinding");
    }
    pub fn register_keybinding_batch(&self, kbs: Vec<Keybinding>) {
        self.sender
            .send(ChanMessage::RegisterKeybindingBatch(kbs))
            .expect("Failed to send RegisterKeybindingBatch");
    }
    pub fn is_running(&self) -> bool {
        self.inner.lock().running.load(Ordering::SeqCst)
    }
    fn make_keybinding_error(keybinding: &Keybinding) -> String {
        let message = format!("Failed to register {:?}.\nAnother running application may already have this binding registered.", &keybinding);
        error!("{}", &message);
        message
    }
    pub fn start(&self, state_arc: Arc<Mutex<AppState>>) {
        let inner = self.inner.clone();
        let receiver = self.receiver.clone();
        let state = state_arc.clone();

        thread::spawn(move || {
            let receiver = receiver.lock();
            {
                let inner = inner.lock();
                inner.register_all(
                    &state
                        .lock()
                        .config
                        .keybindings
                        .iter()
                        .filter(|kb| kb.is_global())
                        .collect(),
                    state.clone(),
                );
            }

            loop {
                if let Ok(msg) = receiver.try_recv() {
                    debug!("KbManager received {:?}", msg);
                    match msg {
                        ChanMessage::ModeCbExecuted => unreachable!(),
                        ChanMessage::Stop => {
                            debug!("Stopping KbManager");
                            let inner = inner.lock();
                            inner.unregister_all();
                            inner.running.store(false, Ordering::SeqCst);
                            break;
                        }
                        ChanMessage::LeaveWorkMode => {
                            let inner = inner.lock();
                            for kb in state
                                .lock()
                                .config
                                .keybindings
                                .iter()
                                .filter(|kb| !kb.is_global())
                            {
                                inner.unregister_kb(kb);
                            }
                        }
                        ChanMessage::EnterWorkMode => {
                            let inner = inner.lock();
                            for kb in state
                                .lock()
                                .config
                                .keybindings
                                .iter()
                                .filter(|kb| !kb.is_global())
                            {
                                inner.register_kb(kb);
                            }
                        }
                        ChanMessage::UnregisterKeybinding(kb) => {
                            let inner = inner.lock();
                            inner.unregister_kb(&kb);
                        }
                        ChanMessage::UnregisterKeybindingBatch(kbs) => {
                            let inner = inner.lock();
                            for kb in kbs {
                                inner.unregister_kb(&kb);
                            }
                        }
                        ChanMessage::RegisterKeybinding(kb) => {
                            let inner = inner.lock();
                            inner.register_kb(&kb);
                        }
                        ChanMessage::RegisterKeybindingBatch(kbs) => {
                            let inner = inner.lock();
                            for kb in kbs {
                                inner.register_kb(&kb);
                            }
                        }
                        ChanMessage::UnregisterKeybindings => {
                            let inner = inner.lock();
                            if state.lock().work_mode {
                                inner.unregister_all();
                            } else {
                                for kb in state
                                    .lock()
                                    .config
                                    .keybindings
                                    .iter()
                                    .filter(|kb| kb.is_global())
                                {
                                    inner.unregister_kb(kb);
                                }
                            }
                            drop(inner);
                        }
                        ChanMessage::RegisterKeybindings => {
                            let inner = inner.lock();
                            let work_mode = state.lock().work_mode;
                            inner.register_all(
                                &state
                                    .lock()
                                    .config
                                    .keybindings
                                    .iter()
                                    .filter(|kb| kb.is_global() || work_mode)
                                    .collect(),
                                state.clone(),
                            );
                        }
                    };
                }

                let inner_lock = inner.lock();
                let kb = do_loop(&inner_lock);
                drop(inner_lock);
                if let Some(kb) = kb {
                    // if we fail to grab state here, the key event will just need to be ignored
                    // to avoid blocking other threads that might be trying to change state.
                    if let Some(state) = state.try_lock_for(Duration::from_millis(100)) {
                        let work_mode = state.work_mode;
                        if work_mode || kb.is_global() {
                            let sender = state.event_channel.sender.clone();
                            sender
                                .send(Event::Keybinding(kb))
                                .expect("Failed to send key event");
                        }
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        });
    }
    pub fn stop(&mut self) {
        if self.inner.lock().stopped.load(Ordering::SeqCst) {
            return;
        }
        self.inner.lock().stopped.store(true, Ordering::SeqCst);
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
fn do_loop(inner: &KbManagerInner) -> Option<Keybinding> {
    use winapi::um::winuser::GetKeyState;
    use winapi::um::winuser::VK_RMENU;
    use winapi::um::winuser::WM_HOTKEY;

    if let Some(msg) = system::win::api::get_current_window_msg() {
        if msg.message != WM_HOTKEY {
            return None;
        }

        if !inner.allow_right_alt {
            // if the right alt key is down skip the hotkey, because we don't support right alt
            // keybindings to avoid false positives
            if unsafe { GetKeyState(VK_RMENU) } & 0b111111110000000 != 0 {
                return None;
            }
        }

        let modifier = Modifier::from_bits((msg.lParam & 0xffff) as u32).unwrap();

        if let Some(key) = Key::from_isize(msg.lParam >> 16) {
            return inner.get_keybinding(key, modifier);
        }
    }

    None
}

pub fn listen(state_arc: Arc<Mutex<AppState>>) -> Sender<()> {
    let (tx, rx) = channel::<()>();

    std::thread::spawn(move || {
        let mut kbs = state_arc.lock().config.keybindings.clone();
        dbg!(&kbs);
        let hook = keyboardhook::start();
        while let Ok(ev) = hook.recv() {
            if rx.try_recv().is_ok() {
                kbs = state_arc.lock().config.keybindings.clone();
            }

            if let InputEvent::KeyDown { key_code, shift, ctrl, win, lalt, ralt } = ev {
                if let Some(key) = Key::from_usize(key_code) {
                    dbg!(&key);
                    for kb in &kbs {
                        if kb.key == key {
                            hook.block(true);
                            continue;
                        }
                    }
                }
            }

            hook.block(false);
        }
    });

    tx
}
