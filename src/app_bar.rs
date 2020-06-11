use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HCURSOR;
use winapi::shared::windef::HICON;
use winapi::shared::windef::HBRUSH;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::winuser::DefWindowProcA;
use winapi::um::winuser::WM_PAINT;
use winapi::um::winuser::PAINTSTRUCT;
use winapi::um::winuser::GetDC;
use winapi::um::winuser::GetClientRect;
use winapi::um::winuser::ReleaseDC;
use winapi::um::winuser::BeginPaint;
use winapi::um::winuser::FillRect;
use winapi::um::winuser::DrawTextA;
use winapi::um::winuser::DT_CENTER;
use winapi::um::winuser::DT_VCENTER;
use winapi::um::winuser::DT_SINGLELINE;
use winapi::um::winuser::EndPaint;
use winapi::um::winuser::RegisterClassA;
use winapi::um::winuser::ShowWindow;
use winapi::um::winuser::SW_SHOW;
use winapi::um::winuser::WNDCLASSA;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::wingdi::SetTextColor;
use winapi::um::wingdi::TRANSPARENT;
use winapi::um::wingdi::SetBkMode;

use crate::CONFIG;

pub static mut HEIGHT: i32 = 0;

unsafe extern "system" fn window_cb(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    println!("Received event: {}", msg);

    if msg == WM_PAINT && WINDOW != 0 {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0
        };

        GetClientRect(WINDOW as HWND, &mut rect);

        let mut paint = PAINTSTRUCT {
            hdc: GetDC(WINDOW as HWND),
            fErase: 0,
            rcPaint: rect,
            fRestore: 0,
            fIncUpdate: 0,
            rgbReserved: [0; 32]
        };

        BeginPaint(WINDOW as HWND, &mut paint);
        EndPaint(WINDOW as HWND, &paint);
    }

    return DefWindowProcA(hwnd, msg, w_param, l_param);
}

static mut WINDOW: i32 = 0;

pub unsafe fn create() {
    HEIGHT = 0;
    let name = "wwm_app_bar";
    let instance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());
    let background_brush = CreateSolidBrush(CONFIG.app_bar_bg as u32);

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
        HEIGHT,
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

pub unsafe fn clear(){
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0
    };

    GetClientRect(WINDOW as HWND, &mut rect);

    FillRect(GetDC(WINDOW as HWND), &mut rect, CreateSolidBrush(CONFIG.app_bar_bg as u32));
}

pub unsafe fn draw_workspace(idx: i32, id: i32){
    if WINDOW != 0 {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0
        };

        GetClientRect(WINDOW as HWND, &mut rect);

        rect.left = rect.left + HEIGHT * idx;
        rect.right = rect.left + HEIGHT;

        let hdc = GetDC(WINDOW as HWND);
        FillRect(hdc, &mut rect, CreateSolidBrush(CONFIG.app_bar_workspace_bg as u32));

        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, 0x00ffffff);

        let id_str = id.to_string();

        DrawTextA(hdc, id_str.as_ptr() as *const i8, id_str.len() as i32, &mut rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
    }
}