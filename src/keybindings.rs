use crate::{event::Event, message_loop, util, CHANNEL, CONFIG, WORK_MODE};
use key::Key;
use keybinding::Keybinding;
use keybinding_type::KeybindingType;
use lazy_static::lazy_static;
use log::{debug, error, info};
use modifier::Modifier;
use num_traits::FromPrimitive;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use winapi::um::winuser::{RegisterHotKey, UnregisterHotKey, WM_HOTKEY};

pub mod key;
pub mod keybinding;
pub mod keybinding_type;
pub mod modifier;

lazy_static! {
    static ref UNREGISTER: AtomicBool = AtomicBool::new(false);
    static ref PREV_MODE: Mutex<Option<String>> = Mutex::new(None);
    pub static ref MODE: Mutex<Option<String>> = Mutex::new(None);
}

fn unregister_keybindings<'a>(keybindings: impl Iterator<Item = &'a Keybinding>) {
    for kb in keybindings {
        info!("Unregistering {:?}", kb);

        unsafe {
            UnregisterHotKey(std::ptr::null_mut(), kb.get_id());
        }
    }
}

fn register_keybindings<'a>(keybindings: impl Iterator<Item = &'a Keybinding>) {
    for kb in keybindings {
        info!("Registering {:?}", kb);

        unsafe {
            util::winapi_nullable_to_result(RegisterHotKey(
                std::ptr::null_mut(),
                kb.get_id(),
                kb.modifier.bits(),
                kb.key as u32,
            ))
            .expect("Failed to register keybinding");
        }
    }
}

fn get_keybinding(keybindings: &[Keybinding], key: Key, modifier: Modifier) -> Option<Keybinding> {
    keybindings
        .iter()
        .find(|kb| kb.key == key && kb.modifier == modifier)
        .cloned()
}

pub fn register() -> Result<(), Box<dyn std::error::Error>> {
    std::thread::spawn(|| {
        let keybindings = CONFIG.lock().unwrap().keybindings.clone();

        while UNREGISTER.load(Ordering::SeqCst) {
            debug!("Waiting for the other thread to get cleaned up");
            // as long as another thread gets unregistered we cant start a new one
            std::thread::sleep(std::time::Duration::from_millis(10))
        }

        register_keybindings(keybindings.iter().filter(|kb| kb.mode == None));

        message_loop::start(|maybe_msg| {
            if UNREGISTER.load(Ordering::SeqCst) {
                debug!("Unregistering hot key manager");
                unregister_keybindings(keybindings.iter().filter(|kb| kb.mode == None));
                UNREGISTER.store(false, Ordering::SeqCst);
                return false;
            }

            let mut prev_mode = PREV_MODE.lock().unwrap();
            let mode = MODE.lock().unwrap().clone();

            if *prev_mode != mode {
                if let Some(mode) = prev_mode.clone() {
                    unregister_keybindings(
                        keybindings
                            .iter()
                            .filter(|kb| kb.mode == Some(mode.clone())),
                    );
                    register_keybindings(keybindings.iter().filter(|kb| {
                        kb.mode == None && kb.typ != KeybindingType::ToggleMode(mode.clone())
                    }));
                }

                if let Some(mode) = mode.clone() {
                    unregister_keybindings(keybindings.iter().filter(|kb| {
                        kb.mode == None && kb.typ != KeybindingType::ToggleMode(mode.clone())
                    }));
                    register_keybindings(
                        keybindings
                            .iter()
                            .filter(|kb| kb.mode == Some(mode.clone())),
                    );
                }

                *prev_mode = mode.clone();
            }

            if let Some(msg) = maybe_msg {
                if msg.message != WM_HOTKEY {
                    return true;
                }

                let work_mode = *WORK_MODE.lock().unwrap();
                let modifier = Modifier::from_bits((msg.lParam & 0xffff) as u32).unwrap();

                if let Some(key) = Key::from_isize(msg.lParam >> 16) {
                    let kb = get_keybinding(&keybindings, key, modifier)
                        .expect("Couldn't find keybinding");

                    if work_mode || kb.typ == KeybindingType::ToggleWorkMode {
                        CHANNEL
                            .sender
                            .clone()
                            .send(Event::Keybinding(kb.clone()))
                            .expect("Failed to send key event");
                    }
                }
            }

            true
        });
    });

    Ok(())
}

pub fn unregister() {
    disable_mode();
    UNREGISTER.store(true, Ordering::SeqCst);
}

pub fn enable_mode(mode: &str) -> bool {
    let mut mode_guard = MODE.lock().unwrap();
    let mode = Some(mode.to_string());

    if *mode_guard == mode {
        return false;
    }

    *mode_guard = mode.clone();

    let sender = CHANNEL.sender.clone();

    let _ = sender
        .send(Event::RedrawAppBar)
        .map_err(|e| error!("{:?}", e));

    true
}

pub fn disable_mode() {
    *MODE.lock().unwrap() = None;

    let sender = CHANNEL.sender.clone();

    let _ = sender
        .send(Event::RedrawAppBar)
        .map_err(|e| error!("{:?}", e));
}
