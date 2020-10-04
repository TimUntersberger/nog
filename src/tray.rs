use std::sync::Arc;

use crate::message_loop;
use crate::util;
use crate::{event::Event, window::Window, window::WindowEvent, AppState};
use lazy_static::lazy_static;
use num_traits::FromPrimitive;
use parking_lot::Mutex;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::minwindef::LOWORD;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HWND;
use winapi::shared::windef::POINT;
use winapi::um::shellapi::Shell_NotifyIconW;
use winapi::um::shellapi::NIF_ICON;
use winapi::um::shellapi::NIF_MESSAGE;
use winapi::um::shellapi::NIF_TIP;
use winapi::um::shellapi::NIM_ADD;
use winapi::um::shellapi::NIM_DELETE;
use winapi::um::shellapi::NOTIFYICONDATAW;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::winuser::CreateIconFromResourceEx;
use winapi::um::winuser::CreatePopupMenu;
use winapi::um::winuser::DefWindowProcW;
use winapi::um::winuser::DestroyMenu;
use winapi::um::winuser::GetCursorPos;
use winapi::um::winuser::InsertMenuW;
use winapi::um::winuser::PostMessageW;
use winapi::um::winuser::RegisterClassA;
use winapi::um::winuser::SendMessageW;
use winapi::um::winuser::SetFocus;
use winapi::um::winuser::SetForegroundWindow;
use winapi::um::winuser::SetMenuItemBitmaps;
use winapi::um::winuser::TrackPopupMenu;
use winapi::um::winuser::LR_DEFAULTCOLOR;
use winapi::um::winuser::MF_BYPOSITION;
use winapi::um::winuser::MF_STRING;
use winapi::um::winuser::TPM_LEFTALIGN;
use winapi::um::winuser::TPM_NONOTIFY;
use winapi::um::winuser::TPM_RETURNCMD;
use winapi::um::winuser::TPM_RIGHTBUTTON;
use winapi::um::winuser::WM_APP;
use winapi::um::winuser::WM_CLOSE;
use winapi::um::winuser::WM_COMMAND;
use winapi::um::winuser::WM_CREATE;
use winapi::um::winuser::WM_INITMENUPOPUP;
use winapi::um::winuser::WM_RBUTTONUP;
use winapi::um::winuser::WNDCLASSA;

pub static WINDOW: Mutex<Option<Window>> = Mutex::new(None);

#[derive(FromPrimitive, Debug, Copy, Clone)]
enum PopupId {
    Exit = 1000,
    Reload = 1001,
}

// unsafe extern "system" fn window_cb(
//     hwnd: HWND,
//     msg: UINT,
//     w_param: WPARAM,
//     l_param: LPARAM,
// ) -> LRESULT {
//     } else if msg == WM_COMMAND {
//         if let Some(id) = PopupId::from_u16(LOWORD(w_param as u32)) {
//             match id {
//                 PopupId::Exit => {
//                     PostMessageW(hwnd, WM_CLOSE, 0, 0);
//                 }
//                 PopupId::Reload => {
//                     CHANNEL
//                         .sender
//                         .clone()
//                         .send(Event::ReloadConfig)
//                         .expect("Failed to send event");
//                 }
//             }
//         }
//     } else if msg == WM_APP && l_param as u32 == WM_RBUTTONUP {
//         SetForegroundWindow(hwnd);
//         show_popup_menu(hwnd);
//         PostMessageW(hwnd, WM_APP + 1, 0, 0);
//     }

//     DefWindowProcW(hwnd, msg, w_param, l_param)
// }

// TODO: use Window struct instead
pub fn create(state: Arc<Mutex<AppState>>) {
    let state_arc = state.clone();
    let state = state_arc.lock();

    let mut window = Window::new()
        .with_title("Nog Tray")
        .with_background_color(state.config.bar.color);

    let sender = state.event_channel.sender.clone();

    drop(state);

    window.create(state_arc, move |event| match event {
        WindowEvent::Create { id, .. } => {
            add_icon(id.to_owned().into());
        }
        WindowEvent::Close { .. } => {
            sender.send(Event::Exit).expect("Failed to send exit event");
        }
        WindowEvent::Native(_) => {}
        _ => {}
    });

    *WINDOW.lock() = Some(window);
}

pub fn add_icon(hwnd: HWND) {
    let icon_bytes = include_bytes!("../assets/logo.png");

    unsafe {
        let icon_handle = CreateIconFromResourceEx(
            icon_bytes.as_ptr() as *mut _,
            icon_bytes.len() as u32,
            1,
            0x00_030_000,
            200,
            200,
            LR_DEFAULTCOLOR,
        );

        let mut tooltip_array = [0u16; 128];
        let version = option_env!("NOG_VERSION")
            .map(|s| format!(" - {}", s))
            .unwrap_or_default();
        let tooltip = format!("Nog{}", version);
        let mut tooltip = tooltip.encode_utf16().collect::<Vec<_>>();
        tooltip.extend(vec![0; 128 - tooltip.len()]);
        tooltip_array.swap_with_slice(&mut tooltip[..]);

        let mut icon_data = NOTIFYICONDATAW::default();
        icon_data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        icon_data.hWnd = hwnd;
        icon_data.uID = 1;
        icon_data.uCallbackMessage = WM_APP;
        icon_data.uFlags = NIF_ICON | NIF_TIP | NIF_MESSAGE;
        icon_data.hIcon = icon_handle;
        icon_data.szTip = tooltip_array;

        Shell_NotifyIconW(NIM_ADD, &mut icon_data);
    }
}

pub fn remove_icon(hwnd: HWND) {
    unsafe {
        let mut icon_data = NOTIFYICONDATAW::default();
        icon_data.hWnd = hwnd;
        icon_data.uID = 1;

        Shell_NotifyIconW(NIM_DELETE, &mut icon_data);
    }
}

unsafe fn show_popup_menu(hwnd: HWND) {
    let menu = CreatePopupMenu();

    let mut exit = util::to_widestring("Exit");
    let mut reload = util::to_widestring("Reload");

    InsertMenuW(
        menu,
        0,
        MF_BYPOSITION | MF_STRING,
        PopupId::Exit as usize,
        exit.as_mut_ptr(),
    );

    InsertMenuW(
        menu,
        0,
        MF_BYPOSITION | MF_STRING,
        PopupId::Reload as usize,
        reload.as_mut_ptr(),
    );

    SetMenuItemBitmaps(
        menu,
        1,
        MF_BYPOSITION,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );

    SetFocus(hwnd);
    SendMessageW(hwnd, WM_INITMENUPOPUP, menu as usize, 0);

    let mut point = POINT::default();
    GetCursorPos(&mut point);

    let cmd = TrackPopupMenu(
        menu,
        TPM_LEFTALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY,
        point.x,
        point.y,
        0,
        hwnd,
        std::ptr::null_mut(),
    );

    SendMessageW(hwnd, WM_COMMAND, cmd as usize, 0);

    DestroyMenu(menu);
}
