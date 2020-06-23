use crate::util;
use crate::CONFIG;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HWND;
use winapi::um::shellapi::Shell_NotifyIconW;
use winapi::um::shellapi::NIF_ICON;
use winapi::um::shellapi::NIF_TIP;
use winapi::um::shellapi::NIM_ADD;
use winapi::um::shellapi::NOTIFYICONDATAW;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::winuser::CreateIconFromResourceEx;
use winapi::um::winuser::DefWindowProcW;
use winapi::um::winuser::DispatchMessageW;
use winapi::um::winuser::GetMessageW;
use winapi::um::winuser::RegisterClassA;
use winapi::um::winuser::TranslateMessage;
use winapi::um::winuser::LR_DEFAULTCOLOR;
use winapi::um::winuser::MSG;
use winapi::um::winuser::WM_CLOSE;
use winapi::um::winuser::WM_CREATE;
use winapi::um::winuser::WNDCLASSA;

unsafe extern "system" fn window_cb(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_CREATE {
        add_icon(hwnd);
    }

    DefWindowProcW(hwnd, msg, w_param, l_param)
}

pub fn create() -> Result<(), util::WinApiResultError> {
    let name = "WWM Tray";

    std::thread::spawn(move || unsafe {
        let instance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());
        let background_brush = CreateSolidBrush(CONFIG.app_bar_bg as u32);

        let class = WNDCLASSA {
            hInstance: instance as HINSTANCE,
            lpszClassName: name.as_ptr() as *const i8,
            lpfnWndProc: Some(window_cb),
            hbrBackground: background_brush,
            ..WNDCLASSA::default()
        };

        RegisterClassA(&class);

        winapi::um::winuser::CreateWindowExA(
            winapi::um::winuser::WS_EX_NOACTIVATE,
            name.as_ptr() as *const i8,
            name.as_ptr() as *const i8,
            0,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            std::ptr::null_mut(),
        );

        let mut msg: MSG = MSG::default();
        while GetMessageW(&mut msg, 0 as HWND, 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });

    Ok(())
}

pub fn add_icon(hwnd: HWND) {
    let icon_bytes = include_bytes!("../logo.png");

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
        let tooltip = "WWM";
        let mut tooltip = tooltip.encode_utf16().collect::<Vec<_>>();
        tooltip.extend(vec![0; 128 - tooltip.len()]);
        tooltip_array.swap_with_slice(&mut tooltip[..]);

        let mut icon_data = NOTIFYICONDATAW::default();
        icon_data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        icon_data.hWnd = hwnd;
        icon_data.uID = 1;
        icon_data.uFlags = NIF_ICON | NIF_TIP;
        icon_data.hIcon = icon_handle;
        icon_data.szTip = tooltip_array;

        Shell_NotifyIconW(NIM_ADD, &mut icon_data);
    }
}
