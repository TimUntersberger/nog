use winapi::shared::windef::HWND;
use winapi::um::winuser::DispatchMessageW;
use winapi::um::winuser::GetMessageW;
use winapi::um::winuser::RegisterHotKey;
use winapi::um::winuser::TranslateMessage;
use winapi::um::winuser::MSG;
use winapi::um::winuser::WM_HOTKEY;

use log::{debug};

use num_traits::FromPrimitive;
use strum_macros::EnumString;

use crate::util;

#[derive(Clone, Copy, FromPrimitive, PartialEq, EnumString, Display)]
#[allow(dead_code)]
pub enum Key {
    Enter = 0x0D,
    Plus = 0xBB,
    Minus = 0xBD,
    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4A,
    K = 0x4B,
    L = 0x4C,
    M = 0x4D,
    N = 0x4E,
    O = 0x4F,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5A,
    #[strum(serialize = "0")]
    Zero = 0x30,
    #[strum(serialize = "1")]
    One = 0x31,
    #[strum(serialize = "2")]
    Two = 0x32,
    #[strum(serialize = "3")]
    Three = 0x33,
    #[strum(serialize = "4")]
    Four = 0x34,
    #[strum(serialize = "5")]
    Five = 0x35,
    #[strum(serialize = "6")]
    Six = 0x36,
    #[strum(serialize = "7")]
    Seven = 0x37,
    #[strum(serialize = "8")]
    Eight = 0x38,
    #[strum(serialize = "9")]
    Nine = 0x39,
}

#[derive(Clone, Copy, EnumString)]
#[allow(dead_code)]
pub enum Modifier {
    Alt = 0x0001,
    Control = 0x0002,
    Shift = 0x0004,
    Win = 0x0008,
}

#[allow(dead_code)]
pub struct HotKey {
    key: Key,
    modifiers: Vec<Modifier>,
    callback: Box<dyn Fn() -> Result<(), Box<dyn std::error::Error>>>,
}

pub struct HotKeyManager {
    hot_keys: Vec<HotKey>,
}

impl HotKeyManager {
    pub fn new() -> Self {
        Self {
            hot_keys: Vec::new(),
        }
    }
    pub fn register_hot_key<F: 'static>(&mut self, key: Key, modifiers: Vec<Modifier>, callback: F) -> Result<(), util::WinApiResultError>
    where
        F: Fn() -> Result<(), Box<dyn std::error::Error>>,
    {
        let mut flags: u32 = 0;

        for modifier in &modifiers {
            flags = flags | *modifier as u32;
        }

        unsafe {
            util::winapi_nullable_to_result(RegisterHotKey(0 as HWND, key as i32, flags, key as u32))?;
        }

        self.hot_keys.push(HotKey {
            key: key,
            modifiers: modifiers,
            callback: Box::new(callback),
        });

        Ok(())
    }
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut msg: MSG = MSG::default();

        unsafe {

            while GetMessageW(&mut msg, 0 as HWND, 0, 0) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);

                if msg.message == WM_HOTKEY {
                    if let Some(key) = Key::from_usize(msg.wParam) {
                        for hot_key in &self.hot_keys {
                            if hot_key.key == key {
                                (hot_key.callback)()?;
                            }
                        }
                    }
                } 
            }
        }

        Ok(())
    }
}
