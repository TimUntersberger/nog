use crate::{event::Event, util, window::Window, window::WindowEvent, AppState};
use num_traits::FromPrimitive;
use parking_lot::Mutex;
use std::sync::Arc;
use winapi::{
    shared::{minwindef::LOWORD, windef::HWND, windef::POINT},
    um::shellapi::Shell_NotifyIconW,
    um::shellapi::NIF_ICON,
    um::shellapi::NIF_MESSAGE,
    um::shellapi::NIF_TIP,
    um::shellapi::NIM_ADD,
    um::shellapi::{NIM_DELETE, NOTIFYICONDATAW},
    um::winuser::CreateIconFromResourceEx,
    um::winuser::CreatePopupMenu,
    um::winuser::DestroyMenu,
    um::winuser::GetCursorPos,
    um::winuser::InsertMenuW,
    um::winuser::PostMessageW,
    um::winuser::SendMessageW,
    um::winuser::SetFocus,
    um::winuser::SetForegroundWindow,
    um::winuser::SetMenuItemBitmaps,
    um::winuser::TrackPopupMenu,
    um::winuser::LR_DEFAULTCOLOR,
    um::winuser::MF_BYPOSITION,
    um::winuser::MF_STRING,
    um::winuser::TPM_LEFTALIGN,
    um::winuser::TPM_NONOTIFY,
    um::winuser::TPM_RETURNCMD,
    um::winuser::TPM_RIGHTBUTTON,
    um::winuser::WM_APP,
    um::winuser::WM_CLOSE,
    um::winuser::WM_COMMAND,
    um::winuser::WM_INITMENUPOPUP,
    um::winuser::WM_RBUTTONUP,
};

pub static WINDOW: Mutex<Option<Window>> = Mutex::new(None);

#[derive(FromPrimitive, Debug, Copy, Clone)]
enum PopupId {
    Exit = 1000,
    Reload = 1001,
}

pub fn create(state: Arc<Mutex<AppState>>) {
    let state_arc = state.clone();
    let state = state_arc.lock();

    let mut window = Window::new()
        .with_title("Nog Tray")
        .with_background_color(state.config.bar.color);

    let sender = state.event_channel.sender.clone();

    drop(state);

    window.create(state_arc, false, move |event| {
        match event {
            WindowEvent::Create { window_id, .. } => {
                add_icon(window_id.to_owned().into());
            }
            WindowEvent::Close { .. } => {
                sender.send(Event::Exit).expect("Failed to send exit event");
            }
            WindowEvent::Native { msg, .. } => {
                if msg.code == WM_COMMAND {
                    if let Some(id) = PopupId::from_u16(LOWORD(msg.params.0 as u32)) {
                        match id {
                            PopupId::Exit => unsafe {
                                PostMessageW(msg.hwnd, WM_CLOSE, 0, 0);
                                sender.send(Event::Exit).expect("Failed to send event");
                            },
                            PopupId::Reload => {
                                sender
                                    .send(Event::ReloadConfig)
                                    .expect("Failed to send event");
                            }
                        }
                    }
                } else if msg.code == WM_APP && msg.params.1 as u32 == WM_RBUTTONUP {
                    unsafe {
                        SetForegroundWindow(msg.hwnd);
                        show_popup_menu(msg.hwnd);
                        PostMessageW(msg.hwnd, WM_APP + 1, 0, 0);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    });

    *WINDOW.lock() = Some(window);
}

pub fn add_icon(hwnd: HWND) {
    let icon_bytes = include_bytes!("../../assets/logo.png");

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
