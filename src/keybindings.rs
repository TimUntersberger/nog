use crate::hot_key_manager::key::Key;
use lazy_static::lazy_static;
use num_traits::FromPrimitive;
use std::{
    collections::VecDeque,
    fmt::Debug,
    sync::atomic::{AtomicBool, Ordering},
};
use winapi::{
    shared::{
        minwindef::{LPARAM, LRESULT, WPARAM},
        windef::HWND,
    },
    um::winuser::{
        CallNextHookEx, DispatchMessageW, PeekMessageW, SetWindowsHookExA, TranslateMessage,
        KBDLLHOOKSTRUCT, MSG, PM_REMOVE, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_MENU, WH_KEYBOARD_LL,
        WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

pub struct KeyPress {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub key: Key,
}

impl Debug for KeyPress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut modifiers: VecDeque<String> = vec![format!("{}", self.key)].into();

        if self.alt {
            modifiers.push_front(String::from("Alt"));
        }

        if self.shift {
            modifiers.push_front(String::from("Shift"));
        }

        if self.control {
            modifiers.push_front(String::from("Control"));
        }

        f.write_str(&modifiers.into_iter().collect::<Vec<String>>().join("+"))
    }
}

lazy_static! {
    static ref SHIFT: AtomicBool = AtomicBool::new(false);
    static ref CONTROL: AtomicBool = AtomicBool::new(false);
    static ref ALT: AtomicBool = AtomicBool::new(false);
}

unsafe extern "system" fn windows_hook_cb(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let data_struct = *(l_param as *mut KBDLLHOOKSTRUCT);
    match w_param as u32 {
        kind if kind == WM_KEYDOWN || kind == WM_SYSKEYDOWN => match data_struct.vkCode as i32 {
            x if x == VK_LSHIFT => {
                SHIFT.store(true, Ordering::SeqCst);
            }
            x if x == VK_LCONTROL => {
                CONTROL.store(true, Ordering::SeqCst);
            }
            x if x == VK_LMENU => {
                ALT.store(true, Ordering::SeqCst);
            }
            _ => {
                if let Some(key) = Key::from_u32(data_struct.vkCode) {
                    let key_press = KeyPress {
                        alt: ALT.load(Ordering::SeqCst),
                        shift: SHIFT.load(Ordering::SeqCst),
                        control: CONTROL.load(Ordering::SeqCst),
                        key,
                    };

                    println!("{:?}", key_press);
                }
            }
        },
        kind if kind == WM_KEYUP || kind == WM_SYSKEYUP => match data_struct.vkCode as i32 {
            x if x == VK_LSHIFT => {
                SHIFT.store(false, Ordering::SeqCst);
            }
            x if x == VK_LCONTROL => {
                CONTROL.store(false, Ordering::SeqCst);
            }
            x if x == VK_LMENU => {
                ALT.store(false, Ordering::SeqCst);
            }
            _ => {}
        },
        _ => {}
    }
    return CallNextHookEx(std::ptr::null_mut(), code, w_param, l_param);
}

pub fn listen() {
    std::thread::spawn(|| unsafe {
        dbg!(SetWindowsHookExA(
            WH_KEYBOARD_LL,
            Some(windows_hook_cb),
            std::ptr::null_mut(),
            0,
        ));

        let mut msg: MSG = MSG::default();
        loop {
            PeekMessageW(&mut msg, 0 as HWND, 0, 0, PM_REMOVE);
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}
