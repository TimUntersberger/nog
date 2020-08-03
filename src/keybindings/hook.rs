use super::{key::Key, key_press::KeyPress, keybinding_type::KeybindingType};
use crate::{CONFIG, WORK_MODE};
use lazy_static::lazy_static;
use num_traits::FromPrimitive;
use std::sync::atomic::{AtomicBool, Ordering, AtomicPtr};
use winapi::{
    shared::{
        minwindef::{LPARAM, LRESULT, WPARAM},
        windef::{HWND, HHOOK__},
    },
    um::winuser::{
        CallNextHookEx, DispatchMessageW, PeekMessageW, SetWindowsHookExA, TranslateMessage,
        KBDLLHOOKSTRUCT, MSG, PM_REMOVE, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN, UnhookWindowsHookEx,
    },
};
use log::error;
use crate::event::Event;

lazy_static! {
    static ref SHIFT: AtomicBool = AtomicBool::new(false);
    static ref CONTROL: AtomicBool = AtomicBool::new(false);
    static ref ALT: AtomicBool = AtomicBool::new(false);
    static ref HOOK: AtomicPtr<HHOOK__> = AtomicPtr::new(std::ptr::null_mut());
}

unsafe extern "system" fn windows_hook_cb(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let data_struct = *(l_param as *mut KBDLLHOOKSTRUCT);
    let kind = w_param as u32;
    let is_down = kind == WM_KEYDOWN || kind == WM_SYSKEYDOWN;

    if let Some(key) = Key::from_u32(data_struct.vkCode) {
        match key {
            Key::LShift => SHIFT.store(is_down, Ordering::SeqCst),
            Key::LControl => CONTROL.store(is_down, Ordering::SeqCst),
            Key::LAlt => ALT.store(is_down, Ordering::SeqCst),
            _ => {
                if is_down {
                    let key_press = KeyPress {
                        alt: ALT.load(Ordering::SeqCst),
                        shift: SHIFT.load(Ordering::SeqCst),
                        control: CONTROL.load(Ordering::SeqCst),
                        key,
                    };

                    let keybindings = CONFIG.lock().unwrap().keybindings.clone();
                    let work_mode = *WORK_MODE.lock().unwrap();

                    for kb in &keybindings {
                        if *kb == key_press {
                            if work_mode || kb.typ == KeybindingType::ToggleWorkMode {
                                let event = Event::Keybinding(kb.clone());

                                crate::CHANNEL.sender.clone().send(event.clone()).map_err(|_| {
                                    error!("Failed to send event {:?}", event);
                                }).unwrap();

                                return 1;
                            }
                        }
                    }
                }
            }
        }
    }

    return CallNextHookEx(std::ptr::null_mut(), code, w_param, l_param);
}
pub fn register() {
    std::thread::spawn(|| unsafe {
        let ptr = SetWindowsHookExA(
            WH_KEYBOARD_LL,
            Some(windows_hook_cb),
            std::ptr::null_mut(),
            0,
        );

        HOOK.store(ptr, Ordering::SeqCst);

        let mut msg: MSG = MSG::default();
        loop {
            PeekMessageW(&mut msg, 0 as HWND, 0, 0, PM_REMOVE);
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

pub fn unregister(){
    let ptr = HOOK.load(Ordering::SeqCst);

    if !ptr.is_null() {
        unsafe {
            UnhookWindowsHookEx(ptr);
        }
        HOOK.store(std::ptr::null_mut(), Ordering::SeqCst);
    }
}
