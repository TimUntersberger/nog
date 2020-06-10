use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HBRUSH;
use winapi::shared::windef::HCURSOR;
use winapi::shared::windef::HICON;
use winapi::shared::windef::HWND;
use winapi::um::winuser::DefWindowProcA;
use winapi::um::winuser::RegisterClassA;
use winapi::um::winuser::ShowWindow;
use winapi::um::winuser::SW_SHOW;
use winapi::um::winuser::WNDCLASSA;
use winapi::um::wingdi::CreateSolidBrush;

unsafe extern "system" fn window_cb(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    return DefWindowProcA(hwnd, msg, w_param, l_param);
}

static mut WINDOW: i32 = 0;

pub unsafe fn create() {
    let name = "wwm_app_bar";
    let instance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());
    let background_brush = CreateSolidBrush(0x0027242c);
    let class = WNDCLASSA {
        hInstance: instance,
        lpszClassName: name.as_ptr() as *const i8,
        lpfnWndProc: Some(window_cb),
        cbClsExtra: 0,
        cbWndExtra: 0,
        style: 0,
        hIcon: 0 as HICON,
        hCursor: 0 as HCURSOR,
        hbrBackground: background_brush,
        lpszMenuName: 0 as *const i8,
    };

    RegisterClassA(&class);

    let window_handle = winapi::um::winuser::CreateWindowExA(
        winapi::um::winuser::WS_EX_NOACTIVATE | winapi::um::winuser::WS_EX_TOPMOST,
        name.as_ptr() as *const i8,
        name.as_ptr() as *const i8,
        winapi::um::winuser::WS_POPUPWINDOW & !winapi::um::winuser::WS_BORDER,
        0,
        0,
        1920,
        20,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        instance,
        std::ptr::null_mut(),
    );

    if window_handle == 0 as HWND {
        println!(
            "window: {} | error: {}",
            window_handle as i32,
            winapi::um::errhandlingapi::GetLastError() as i32
        );
    } else {
        println!("window: {}", window_handle as i32);
        WINDOW = window_handle as i32;
        ShowWindow(window_handle, SW_SHOW);
    }
}
