use crate::change_workspace;
use crate::display::Display;
use crate::event::Event;
use crate::tile_grid::TileGrid;
use crate::CHANNEL;
use crate::DISPLAY;
use crate::GRIDS;
use crate::WORKSPACE_ID;
use lazy_static::lazy_static;
use log::{debug, info};
use std::sync::Mutex;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HBRUSH;
use winapi::shared::windef::HDC;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::shared::windef::SIZE;
use winapi::shared::windowsx::GET_X_LPARAM;
use winapi::um::wingdi::CreateFontIndirectA;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::wingdi::GetTextExtentPoint32A;
use winapi::um::wingdi::SelectObject;
use winapi::um::wingdi::SetBkColor;
use winapi::um::wingdi::SetBkMode;
use winapi::um::wingdi::SetTextColor;
use winapi::um::wingdi::LOGFONTA;
use winapi::um::wingdi::TRANSPARENT;
use winapi::um::winuser::BeginPaint;
use winapi::um::winuser::DefWindowProcA;
use winapi::um::winuser::DispatchMessageW;
use winapi::um::winuser::DrawTextA;
use winapi::um::winuser::EndPaint;
use winapi::um::winuser::FillRect;
use winapi::um::winuser::GetClientRect;
use winapi::um::winuser::GetDC;
use winapi::um::winuser::GetMessageW;
use winapi::um::winuser::LoadCursorA;
use winapi::um::winuser::RegisterClassA;
use winapi::um::winuser::SendMessageA;
use winapi::um::winuser::SetCursor;
use winapi::um::winuser::ShowWindow;
use winapi::um::winuser::TranslateMessage;
use winapi::um::winuser::DT_CENTER;
use winapi::um::winuser::DT_SINGLELINE;
use winapi::um::winuser::DT_VCENTER;
use winapi::um::winuser::IDC_ARROW;
use winapi::um::winuser::MSG;
use winapi::um::winuser::PAINTSTRUCT;
use winapi::um::winuser::SW_HIDE;
use winapi::um::winuser::SW_SHOW;
use winapi::um::winuser::WM_CLOSE;
use winapi::um::winuser::WM_CREATE;
use winapi::um::winuser::WM_LBUTTONDOWN;
use winapi::um::winuser::WM_PAINT;
use winapi::um::winuser::WM_SETCURSOR;
use winapi::um::winuser::WNDCLASSA;

use std::ffi::CString;

use crate::util;
use crate::CONFIG;

lazy_static! {
    pub static ref HEIGHT: Mutex<i32> = Mutex::new(0);
    pub static ref WINDOW: Mutex<i32> = Mutex::new(0);
    pub static ref FONT: Mutex<i32> = Mutex::new(0);
    pub static ref REDRAW_REASON: Mutex<RedrawAppBarReason> = Mutex::new(RedrawAppBarReason::Time);
}

#[derive(Copy, Clone, Debug)]
pub enum RedrawAppBarReason {
    Time,
    Workspace,
}

unsafe extern "system" fn window_cb(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_CLOSE {
        *WINDOW.lock().unwrap() = 0;
    } else if msg == WM_SETCURSOR {
        SetCursor(LoadCursorA(std::ptr::null_mut(), IDC_ARROW as *const i8)); // Force a normal cursor. This probably shouldn't be done this way but whatever
    } else if msg == WM_LBUTTONDOWN {
        info!("Received mouse click");
        let x = GET_X_LPARAM(l_param);
        let id = x / CONFIG.app_bar_height + 1;

        if id <= 10 {
            let mut grids = GRIDS.lock().unwrap();
            let grid = grids
                .iter_mut()
                .find(|g| g.id == id)
                .unwrap();

            if !grid.tiles.is_empty() || grid.visible {
                drop(grid);
                drop(grids);
                change_workspace(id).expect("Failed to change workspace");
            }
        }
    } else if msg == WM_CREATE {
        info!("loading font");
        load_font();
    } else if !hwnd.is_null() && msg == WM_PAINT {
            info!("Received paint");
            let reason = *REDRAW_REASON.lock().unwrap();
            debug!("Reason for paint was {:?}", reason);
            let mut paint = PAINTSTRUCT::default();

            GetClientRect(hwnd, &mut paint.rcPaint);

            BeginPaint(hwnd, &mut paint);

            match reason {
                RedrawAppBarReason::Time => {
                    draw_datetime(hwnd).expect("Failed to draw datetime");
                }
                RedrawAppBarReason::Workspace => {
                    draw_workspaces(hwnd);
                }
            }

            EndPaint(hwnd, &paint);
    }

    DefWindowProcA(hwnd, msg, w_param, l_param)
}

pub fn redraw(reason: RedrawAppBarReason) {
    unsafe {
        let hwnd = *WINDOW.lock().unwrap() as HWND;

        if hwnd == 0 as HWND {
            return;
        }

        *REDRAW_REASON.lock().unwrap() = reason;

        //TODO: handle error
        SendMessageA(hwnd, WM_PAINT, 0, 0);
    }
}

fn draw_workspaces(_hwnd: HWND) {
    let id = *WORKSPACE_ID.lock().unwrap();

    let grids = GRIDS.lock().unwrap();

    let workspaces: Vec<&TileGrid> = grids
        .iter()
        .filter(|g| !g.tiles.is_empty() || g.id == id)
        .collect();

    //erase last workspace
    debug!("Erasing {}", workspaces.len());
    erase_workspace((workspaces.len()) as i32);
    for (i, workspace) in workspaces.iter().enumerate() {
        draw_workspace(i as i32, workspace.id, workspace.id == id).expect("Failed to draw workspace");
    }
}

fn erase_workspace(id: i32) {
    unsafe {
        let mut rect = RECT::default();
        let hwnd = *WINDOW.lock().unwrap() as HWND;
        let hdc = GetDC(hwnd);
        GetClientRect(hwnd, &mut rect);

        rect.left += CONFIG.app_bar_height * id;
        rect.right = rect.left + CONFIG.app_bar_height;

        FillRect(hdc, &rect, CreateSolidBrush(CONFIG.app_bar_bg as u32));
    }
}

pub fn set_font(dc: HDC) {
    unsafe {
        SelectObject(dc, *FONT.lock().unwrap() as *mut std::ffi::c_void);
    }
}

pub fn load_font() {
    unsafe {
        let mut logfont = LOGFONTA::default();
        let mut font_name: [i8; 32] = [0; 32];

        for (i, byte) in CString::new(CONFIG.app_bar_font.as_str())
            .unwrap()
            .as_bytes()
            .iter()
            .enumerate()
        {
            font_name[i] = *byte as i8;
        }

        logfont.lfHeight = CONFIG.app_bar_font_size;
        logfont.lfFaceName = font_name;

        let font = CreateFontIndirectA(&logfont) as i32;

        debug!("Using font {}", font);

        *FONT.lock().unwrap() = font;
    }
}

pub fn create(display: &Display) -> Result<(), util::WinApiResultError> {
    info!("Creating appbar");
    let name = "wwm_app_bar";
    let mut height_guard = HEIGHT.lock().unwrap();

    *height_guard = CONFIG.app_bar_height;

    let height = *height_guard;
    let display_width = display.width;

    std::thread::spawn(|| loop {
        std::thread::sleep(std::time::Duration::from_millis(950));
        if *WINDOW.lock().unwrap() == 0 {
            break;
        }
        CHANNEL
            .sender
            .clone()
            .send(Event::RedrawAppBar(RedrawAppBarReason::Time)).expect("Failed to send redraw-app-bar event");
    });

    std::thread::spawn(move || unsafe {
        //TODO: Handle error
        let instance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());
        //TODO: Handle error
        let background_brush = CreateSolidBrush(CONFIG.app_bar_bg as u32);

        let class = WNDCLASSA {
            hInstance: instance as HINSTANCE,
            lpszClassName: name.as_ptr() as *const i8,
            lpfnWndProc: Some(window_cb),
            hbrBackground: background_brush as HBRUSH,
            ..WNDCLASSA::default()
        };

        RegisterClassA(&class);

        //TODO: handle error
        let window_handle = winapi::um::winuser::CreateWindowExA(
            winapi::um::winuser::WS_EX_NOACTIVATE | winapi::um::winuser::WS_EX_TOPMOST,
            name.as_ptr() as *const i8,
            name.as_ptr() as *const i8,
            winapi::um::winuser::WS_POPUPWINDOW & !winapi::um::winuser::WS_BORDER,
            0,
            0,
            display_width,
            height,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance as HINSTANCE,
            std::ptr::null_mut(),
        );

        *WINDOW.lock().unwrap() = window_handle as i32;

        show();

        let mut msg: MSG = MSG::default();
        while GetMessageW(&mut msg, window_handle, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });

    Ok(())
}

pub fn close() {
    unsafe {
        info!("Closing appbar");
        let hwnd = *WINDOW.lock().unwrap();
        SendMessageA(hwnd as HWND, WM_CLOSE, 0, 0);
    }
}

#[allow(dead_code)]
pub fn hide() {
    unsafe {
        let hwnd = *WINDOW.lock().unwrap(); // Need to eager evaluate else there is a deadlock
        ShowWindow(hwnd as HWND, SW_HIDE);
    }
}

pub fn show() {
    let hwnd = *WINDOW.lock().unwrap() as HWND; // Need to eager evaluate else there is a deadlock

    unsafe {
        ShowWindow(hwnd, SW_SHOW);
    }

    draw_workspaces(hwnd);
    draw_datetime(hwnd).expect("Failed to draw datetime");
}

pub fn draw_datetime(hwnd: HWND) -> Result<(), util::WinApiResultError> {
    if !hwnd.is_null() {
        let mut rect = RECT::default();

        unsafe {
            debug!("Getting the rect for the appbar");
            util::winapi_nullable_to_result(GetClientRect(hwnd, &mut rect))?;
            let text = format!("{}", chrono::Local::now().format("%T"));
            let text_len = text.len() as i32;
            let c_text = CString::new(text).unwrap();
            let display = DISPLAY.lock().unwrap();

            debug!("Getting the device context");
            let hdc = util::winapi_ptr_to_result(GetDC(hwnd))?;

            set_font(hdc);

            let mut size = SIZE::default();

            util::winapi_nullable_to_result(GetTextExtentPoint32A(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut size,
            ))?;

            rect.left = display.width / 2 - (size.cx / 2) - 10;
            rect.right = display.width / 2 + (size.cx / 2) + 10;

            debug!("Setting the text color");
            //TODO: handle error
            SetTextColor(hdc, 0x00ffffff);

            debug!("Setting the background color");
            SetBkColor(hdc, CONFIG.app_bar_bg as u32);

            debug!("Writing the time");
            util::winapi_nullable_to_result(DrawTextA(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            ))?;

            let text = format!("{}", chrono::Local::now().format("%e %b %Y"));
            let text_len = text.len() as i32;
            let c_text = CString::new(text).unwrap();

            util::winapi_nullable_to_result(GetTextExtentPoint32A(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut size,
            ))?;

            rect.right = display.width - 10;
            rect.left = rect.right - size.cx;

            debug!("Writing the date");
            util::winapi_nullable_to_result(DrawTextA(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            ))?;
        }
    }

    Ok(())
}

pub fn draw_workspace(idx: i32, id: i32, focused: bool) -> Result<(), util::WinApiResultError> {
    let window = *WINDOW.lock().unwrap() as HWND;
    if !window.is_null() {
        let mut rect = RECT::default();
        let height = *HEIGHT.lock().unwrap();

        unsafe {
            debug!("Getting the rect for the appbar");
            util::winapi_nullable_to_result(GetClientRect(window, &mut rect))?;

            rect.left += height * idx;
            rect.right = rect.left + height;

            debug!("Getting the device context");
            let hdc = util::winapi_ptr_to_result(GetDC(window))?;

            set_font(hdc);

            debug!("Setting the background to transparent");
            SetBkMode(hdc, TRANSPARENT as i32);

            debug!("Setting the text color");
            //TODO: handle error
            let bg_color = if focused {
                SetTextColor(hdc, CONFIG.app_bar_workspace_bg as u32);
                0x00ffffff
            } else {
                SetTextColor(hdc, 0x00ffffff);
                CONFIG.app_bar_workspace_bg
            };

            debug!("Drawing background");
            FillRect(hdc, &rect, CreateSolidBrush(bg_color as u32));

            let id_str = id.to_string();
            let len = id_str.len() as i32;
            let id_cstr = CString::new(id_str).unwrap();

            debug!("Writing the text");
            util::winapi_nullable_to_result(DrawTextA(
                hdc,
                id_cstr.as_ptr(),
                len,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            ))?;
        }
    }

    Ok(())
}
