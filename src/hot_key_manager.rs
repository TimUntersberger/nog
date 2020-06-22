use winapi::shared::windef::HWND;
use winapi::um::winuser::DispatchMessageW;
use winapi::um::winuser::GetMessageW;
use winapi::um::winuser::RegisterHotKey;
use winapi::um::winuser::TranslateMessage;
use winapi::um::winuser::MSG;
use winapi::um::winuser::WM_HOTKEY;
use bitflags::bitflags;
use log::{debug};

use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use strum_macros::EnumString;

use crate::util;

#[derive(Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, EnumString, Display)]
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
    Left = 0x25,
    Up = 0x26,
    Right = 0x27,
    Down = 0x28,
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

bitflags!{
    #[derive(Default)]
    #[allow(dead_code)]
    pub struct Modifier: u32 {
        const ALT = 0x0001;
        const CONTROL = 0x0002;
        const SHIFT = 0x0004;
        const WIN = 0x0008;
    }
}

#[allow(dead_code)]
pub struct HotKey {
    key: Key,
    modifier: Modifier,
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
    pub fn register_hot_key<F: 'static>(&mut self, key: Key, modifier: Modifier, callback: F, id: i32) -> Result<(), util::WinApiResultError>
    where
        F: Fn() -> Result<(), Box<dyn std::error::Error>>,
    {
        unsafe {
            util::winapi_nullable_to_result(RegisterHotKey(0 as HWND, id, modifier.bits(), key as u32))?;
        }

        self.hot_keys.push(HotKey {
            key: key,
            modifier,
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
                    let modifier = Modifier::from_bits((msg.lParam & 0xffff) as u32).unwrap();

                    if let Some(key) = Key::from_isize(msg.lParam >> 16) {
                        for hot_key in &self.hot_keys {
                            if hot_key.key == key && modifier == hot_key.modifier {
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
