pub mod key;
pub mod modifier;

use crate::CONFIG;
use key::Key;
use modifier::Modifier;

use winapi::shared::windef::HWND;
use winapi::um::winuser::DispatchMessageW;
use winapi::um::winuser::GetMessageW;
use winapi::um::winuser::RegisterHotKey;
use winapi::um::winuser::TranslateMessage;
use winapi::um::winuser::MSG;
use winapi::um::winuser::WM_HOTKEY;
use log::{info};

use strum_macros::EnumString;
use num_traits::FromPrimitive;

use crate::CHANNEL;
use crate::tile_grid::SplitDirection;
use crate::util;
use crate::event::Event;

pub type Command = String;

#[derive(Clone, Copy, EnumString, PartialEq, Debug)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down
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
    Split(SplitDirection),
}

#[derive(Debug, Clone)]
pub struct Keybinding {
    pub typ: KeybindingType,
    pub key: Key,
    pub modifier: Modifier
}

pub fn register() -> Result<(), Box<dyn std::error::Error>> {
    let keybindings = CONFIG.keybindings.clone();

    std::thread::spawn(move || {
        let mut msg: MSG = MSG::default();

        for kb in &keybindings {
            let key = kb.key as u32;
            let modifier = kb.modifier.bits();
            let id = key + modifier;

            info!(
                "Registering Keybinding({}+{}, {})",
                format!("{:?}", kb.modifier).replace(" | ", "+"),
                kb.key,
                kb.typ
            );

            unsafe {
                util::winapi_nullable_to_result(RegisterHotKey(0 as HWND, id as i32, modifier, key)).unwrap();
            }
        }

        unsafe {
            while GetMessageW(&mut msg, 0 as HWND, 0, 0) != 0 {
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
        }
    });

    Ok(())
}