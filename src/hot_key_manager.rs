use winapi::um::winuser::WM_HOTKEY;
use winapi::um::winuser::DispatchMessageW;
use winapi::um::winuser::TranslateMessage;
use winapi::um::winuser::GetMessageW;
use winapi::um::winuser::MSG;
use winapi::um::winuser::RegisterHotKey;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::minwindef::UINT;
use winapi::shared::windef::POINT;
use winapi::shared::windef::HWND;
use num_traits::FromPrimitive;

#[derive(Clone, Copy, FromPrimitive, PartialEq)]
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
    Z = 0x5A 
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Modifier {
    Alt = 0x0001,
    Control = 0x0002,
    Shift = 0x0004,
    Win = 0x0008
}

pub struct HotKey {
    key: Key,
    modifiers: Vec<Modifier>,
    callback: Box<dyn Fn()>
}

pub struct HotKeyManager {
    hot_keys: Vec<HotKey>
}

impl HotKeyManager {
    pub fn new() -> Self {
        Self {
            hot_keys: Vec::new()
        }
    }
    pub unsafe fn register_hot_key<F: 'static>(&mut self, key: Key, modifiers: Vec<Modifier>, callback: F) where F: Fn() {
        let mut flags: u32 = 0;

        for modifier in &modifiers {
            flags = flags | *modifier as u32;
        }

        RegisterHotKey(
            0 as HWND,
            key as i32,
            flags,
            key as u32
        );

        self.hot_keys.push(HotKey {
            key: key,
            modifiers: modifiers,
            callback: Box::new(callback)
        });
    }
    pub unsafe fn start(&self){
        let mut msg: MSG = MSG {
			hwnd : 0 as HWND,
			message : 0 as UINT,
			wParam : 0 as WPARAM,
			lParam : 0 as LPARAM,
			time : 0 as DWORD,
			pt : POINT { x: 0, y: 0, },
        };

        loop {
            if GetMessageW(&mut msg, 0 as HWND, WM_HOTKEY, WM_HOTKEY) == 0 {
                break;
            }

            TranslateMessage(&msg);
            DispatchMessageW(&msg);

            if msg.message == WM_HOTKEY {
                if let Some(key) = Key::from_usize(msg.wParam) {
                    for hot_key in &self.hot_keys {
                        if hot_key.key == key {
                            (hot_key.callback)();
                        }
                    }
                }
            }
        }
    }
}