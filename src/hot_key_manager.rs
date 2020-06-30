use crate::event::Event;
use crate::tile_grid::SplitDirection;
use crate::util;
use crate::CHANNEL;
use crate::CONFIG;
use crate::WORK_MODE;
use key::Key;
use lazy_static::lazy_static;
use log::{debug, info};
use modifier::Modifier;
use num_traits::FromPrimitive;
use std::sync::Mutex;
use strum_macros::EnumString;
use winapi::shared::windef::HWND;
use winapi::um::winuser::DispatchMessageW;
use winapi::um::winuser::PeekMessageW;
use winapi::um::winuser::RegisterHotKey;
use winapi::um::winuser::TranslateMessage;
use winapi::um::winuser::UnregisterHotKey;
use winapi::um::winuser::MSG;
use winapi::um::winuser::PM_REMOVE;
use winapi::um::winuser::WM_HOTKEY;

pub mod key;
pub mod modifier;

pub type Command = String;

#[derive(Clone, Copy, EnumString, PartialEq, Debug)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Display, Debug, Clone, PartialEq)]
pub enum KeybindingType {
    CloseTile,
    Quit,
    ChangeWorkspace(i32),
    ToggleFloatingMode,
    ToggleWorkMode,
    ToggleFullscreen,
    Launch(Command),
    Focus(Direction),
    Swap(Direction),
    MoveToWorkspace(i32),
    Split(SplitDirection),
}

#[derive(Debug, Clone)]
pub struct Keybinding {
    pub typ: KeybindingType,
    pub key: Key,
    pub modifier: Modifier,
    pub registered: bool,
}

lazy_static! {
    static ref UNREGISTER: Mutex<bool> = Mutex::new(false);
}

fn unregister_keybindings<'a>(keybindings: impl Iterator<Item = &'a mut Keybinding>) {
    for kb in keybindings {
        if kb.registered {
            let key = kb.key as u32;
            let modifier = kb.modifier.bits();
            let id = key + modifier;

            kb.registered = false;

            info!(
                "Unregistering Keybinding({}+{}, {})",
                format!("{:?}", kb.modifier).replace(" | ", "+"),
                kb.key,
                kb.typ
            );

            unsafe {
                UnregisterHotKey(0 as HWND, id as i32);
            }
        }
    }
}

fn register_keybindings<'a>(keybindings: impl Iterator<Item = &'a mut Keybinding>) {
    for kb in keybindings {
        if !kb.registered {
            let key = kb.key as u32;
            let modifier = kb.modifier.bits();
            let id = key + modifier;

            kb.registered = true;

            info!(
                "Registering Keybinding({}+{}, {})",
                format!("{:?}", kb.modifier).replace(" | ", "+"),
                kb.key,
                kb.typ
            );

            unsafe {
                util::winapi_nullable_to_result(RegisterHotKey(
                    0 as HWND, id as i32, modifier, key,
                ))
                .expect("Failed to register keybinding");
            }
        }
    }
}

pub fn register() -> Result<(), Box<dyn std::error::Error>> {
    std::thread::spawn(|| {
        let mut keybindings = CONFIG.lock().unwrap().keybindings.clone();
        let mut msg: MSG = MSG::default();

        while *UNREGISTER.lock().unwrap() {
            debug!("Waiting for other thread get cleaned up");
            // as long as another thread gets unregistered we cant start a new one
            std::thread::sleep(std::time::Duration::from_millis(10))
        }

        if *WORK_MODE.lock().unwrap() {
            register_keybindings(keybindings.iter_mut());
        } else {
            register_keybindings(
                keybindings
                    .iter_mut()
                    .filter(|kb| kb.typ == KeybindingType::ToggleWorkMode),
            );
        }

        unsafe {
            loop {
                if *UNREGISTER.lock().unwrap() {
                    debug!("Unregistering hot key manager");
                    unregister_keybindings(keybindings.iter_mut());
                    *UNREGISTER.lock().unwrap() = false;
                    break;
                }

                while PeekMessageW(&mut msg, 0 as HWND, 0, 0, PM_REMOVE) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);

                    if msg.message == WM_HOTKEY {
                        let modifier = Modifier::from_bits((msg.lParam & 0xffff) as u32).unwrap();

                        if let Some(key) = Key::from_isize(msg.lParam >> 16) {
                            for kb in &keybindings {
                                if kb.key == key && kb.modifier == modifier {
                                    CHANNEL
                                        .sender
                                        .clone()
                                        .send(Event::Keybinding(kb.clone()))
                                        .expect("Failed to send key event");
                                }
                            }
                        }
                    }
                }

                let work_mode = *WORK_MODE.lock().unwrap();
                if !work_mode {
                    unregister_keybindings(
                        keybindings
                            .iter_mut()
                            .filter(|kb| kb.typ != KeybindingType::ToggleWorkMode),
                    );
                } else {
                    register_keybindings(keybindings.iter_mut());
                }

                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
    });

    Ok(())
}

pub fn unregister() {
    *UNREGISTER.lock().unwrap() = true;
}
