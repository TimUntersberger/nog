pub mod key;
pub mod modifier;

use std::sync::Mutex;
use crate::event::Event;
use crate::tile_grid::SplitDirection;
use crate::util;
use crate::CHANNEL;
use crate::CONFIG;
use key::Key;
use lazy_static::lazy_static;
use log::info;
use modifier::Modifier;
use num_traits::FromPrimitive;
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
    static ref WORK_MODE: Mutex<bool> = Mutex::new(false);
}

pub fn enable(){
    *WORK_MODE.lock().unwrap() = true;
}

pub fn disable(){
    *WORK_MODE.lock().unwrap() = false;
}

pub fn register() -> Result<(), Box<dyn std::error::Error>> {
    if CONFIG.work_mode {
        enable();
    }
    std::thread::spawn(|| {
        let mut keybindings = CONFIG.keybindings.clone();
        let mut msg: MSG = MSG::default();

        let work_mode = WORK_MODE.lock().unwrap();

        for kb in &mut keybindings {
            if *work_mode || kb.typ == KeybindingType::ToggleWorkMode {
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
                    .unwrap();
                }
            }
        }

        drop(work_mode);

        unsafe {
            loop {
                while PeekMessageW(&mut msg, 0 as HWND, 0, 0, PM_REMOVE) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);

                    if msg.message == WM_HOTKEY {
                        let modifier = Modifier::from_bits((msg.lParam & 0xffff) as u32).unwrap();

                        if let Some(key) = Key::from_isize(msg.lParam >> 16) {
                            for kb in &keybindings {
                                if kb.key == key && kb.modifier == modifier {
                                    CHANNEL.sender.clone().send(Event::Keybinding(kb.clone()));
                                }
                            }
                        }
                    }
                }

                if !*WORK_MODE.lock().unwrap() {
                    for kb in &mut keybindings {
                        if kb.registered && kb.typ != KeybindingType::ToggleWorkMode {
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

                            UnregisterHotKey(0 as HWND, id as i32);
                        }
                    }
                } else {
                    for kb in &mut keybindings {
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

                            RegisterHotKey(0 as HWND, id as i32, modifier, key);
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    });

    Ok(())
}
