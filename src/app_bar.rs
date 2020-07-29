use crate::change_workspace;
use crate::display::get_display_by_hmonitor;
use crate::event::Event;
use crate::is_visible_workspace;
use crate::tile_grid::TileGrid;
use crate::util;
use crate::CHANNEL;
use crate::CONFIG;
use crate::DISPLAYS;
use crate::GRIDS;
use crate::WORKSPACE_ID;
use lazy_static::lazy_static;
use log::{debug, error, info};
use std::collections::HashMap;
use std::ffi::CString;
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
use winapi::um::wingdi::DeleteObject;
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
use winapi::um::winuser::ReleaseDC;
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
use winapi::um::winuser::{UnregisterClassA, WNDCLASSA};

lazy_static! {
    pub static ref HEIGHT: Mutex<i32> = Mutex::new(0);
    //HMONITOR, HWND
    pub static ref WINDOWS: Mutex<HashMap<i32, i32>> = Mutex::new(HashMap::new());
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
        WINDOWS.lock().unwrap().remove(&(hwnd as i32));
    } else if msg == WM_SETCURSOR {
        // Force a normal cursor. This probably shouldn't be done this way but whatever
        SetCursor(LoadCursorA(std::ptr::null_mut(), IDC_ARROW as *const i8));
    } else if msg == WM_LBUTTONDOWN {
        info!("Received mouse click");
        let x = GET_X_LPARAM(l_param);
        let id = x / CONFIG.lock().unwrap().app_bar_height + 1;

        if id <= 10 {
            let mut grids = GRIDS.lock().unwrap();
            let grid = grids.iter_mut().find(|g| g.id == id).unwrap();

            if !grid.tiles.is_empty() || is_visible_workspace(id) {
                drop(grids);
                change_workspace(id).expect("Failed to change workspace");
            }
        }
    } else if msg == WM_CREATE {
        info!("loading font");
        load_font();
    } else if msg == WM_PAINT {
        let now = std::time::SystemTime::now();
        let reason = *REDRAW_REASON.lock().unwrap();
        let mut paint = PAINTSTRUCT::default();

        GetClientRect(hwnd, &mut paint.rcPaint);

        BeginPaint(hwnd, &mut paint);

        match reason {
            RedrawAppBarReason::Time => {
                if draw_datetime(hwnd).is_err() {
                    error!("Failed to draw datetime");
                }
            }
            RedrawAppBarReason::Workspace => {
                debug!("Received redraw-app-bar event");
                draw_workspaces(hwnd);
                debug!(
                    "Painting workspaces took {}ms",
                    now.elapsed().expect("Failed to get systemtime").as_millis()
                )
            }
        }

        EndPaint(hwnd, &paint);
    }

    DefWindowProcA(hwnd, msg, w_param, l_param)
}

pub fn redraw(reason: RedrawAppBarReason) {
    unsafe {
        *REDRAW_REASON.lock().unwrap() = reason;

        let hwnds: Vec<i32> = WINDOWS
            .lock()
            .unwrap()
            .iter()
            .map(|(_, hwnd)| *hwnd)
            .collect();

        for hwnd in hwnds {
            //TODO: handle error
            SendMessageA(hwnd as HWND, WM_PAINT, 0, 0);
        }
    }
}

fn draw_workspaces(hwnd: HWND) {
    let grids = GRIDS.lock().unwrap();

    let monitor = *WINDOWS
        .lock()
        .unwrap()
        .iter()
        .find(|(_, v)| **v == hwnd as i32)
        .map(|(m, _)| m)
        .expect("Couldn't find monitor for appbar");

    debug!("On monitor {}", monitor as i32);

    let workspaces: Vec<&TileGrid> = grids
        .iter()
        .filter(|g| {
            (!g.tiles.is_empty() || is_visible_workspace(g.id)) && g.display.hmonitor == monitor
        })
        .collect();

    //erase last workspace
    erase_workspace(hwnd, (workspaces.len()) as i32);

    for (i, workspace) in workspaces.iter().enumerate() {
        draw_workspace(
            hwnd,
            i as i32,
            workspace.id,
            *WORKSPACE_ID.lock().unwrap() == workspace.id,
        )
        .expect("Failed to draw workspace");
    }
}

fn erase_workspace(hwnd: HWND, id: i32) {
    unsafe {
        let mut rect = RECT::default();
        let app_bar_height = CONFIG.lock().unwrap().app_bar_height;
        let app_bar_bg = CONFIG.lock().unwrap().app_bar_bg;
        let brush = CreateSolidBrush(app_bar_bg as u32);

        let hdc = GetDC(hwnd);
        GetClientRect(hwnd, &mut rect);

        rect.left += app_bar_height * id;
        rect.right = rect.left + app_bar_height;

        FillRect(hdc, &rect, brush);

        DeleteObject(brush as *mut std::ffi::c_void);
    }
}

pub fn set_font(dc: HDC) {
    unsafe {
        SelectObject(dc, *FONT.lock().unwrap() as *mut std::ffi::c_void);
    }
}

pub fn load_font() {
    if *FONT.lock().unwrap() != 0 {
        return;
    }
    unsafe {
        let mut logfont = LOGFONTA::default();
        let mut font_name: [i8; 32] = [0; 32];
        let app_bar_font = CONFIG.lock().unwrap().app_bar_font.clone();
        let app_bar_font_size = CONFIG.lock().unwrap().app_bar_font_size;

        for (i, byte) in CString::new(app_bar_font.as_str())
            .unwrap()
            .as_bytes()
            .iter()
            .enumerate()
        {
            font_name[i] = *byte as i8;
        }

        logfont.lfHeight = app_bar_font_size;
        logfont.lfFaceName = font_name;

        let font = CreateFontIndirectA(&logfont) as i32;

        debug!("Using font {}", font);

        *FONT.lock().unwrap() = font;
    }
}

pub fn create() -> Result<(), util::WinApiResultError> {
    info!("Creating appbar");

    let name = "wwm_app_bar";

    let mut height_guard = HEIGHT.lock().unwrap();

    let app_bar_bg = CONFIG.lock().unwrap().app_bar_bg;

    *height_guard = CONFIG.lock().unwrap().app_bar_height;

    let height = *height_guard;

    std::thread::spawn(|| loop {
        std::thread::sleep(std::time::Duration::from_millis(950));
        if WINDOWS.lock().unwrap().is_empty() {
            break;
        }

        CHANNEL
            .sender
            .clone()
            .send(Event::RedrawAppBar(RedrawAppBarReason::Time))
            .expect("Failed to send redraw-app-bar event");
    });

    for display in DISPLAYS.lock().unwrap().clone() {
        std::thread::spawn(move || unsafe {
            if WINDOWS
                .lock()
                .unwrap()
                .contains_key(&(display.hmonitor as i32))
            {
                error!(
                    "Appbar for monitor {} already exists. Aborting",
                    display.hmonitor as i32
                );
            }

            debug!("Creating appbar for display {}", display.hmonitor as i32);

            let display_width = display.width();
            //TODO: Handle error
            let instance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());
            //TODO: Handle error
            let background_brush = CreateSolidBrush(app_bar_bg as u32);

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
                display.left,
                display.top,
                display_width,
                height,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance as HINSTANCE,
                std::ptr::null_mut(),
            );

            WINDOWS
                .lock()
                .unwrap()
                .insert(display.hmonitor as i32, window_handle as i32);

            ShowWindow(window_handle, SW_SHOW);
            draw_workspaces(window_handle);
            draw_datetime(window_handle).expect("Failed to draw datetime");

            let mut msg: MSG = MSG::default();
            while GetMessageW(&mut msg, window_handle, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        });
    }

    Ok(())
}

pub fn close() {
    unsafe {
        info!("Closing appbar");

        let windows: Vec<(i32, i32)> = WINDOWS
            .lock()
            .unwrap()
            .iter()
            .map(|(hmonitor, hwnd)| (*hmonitor, *hwnd))
            .collect();

        for (hmonitor, hwnd) in windows {
            SendMessageA(hwnd as HWND, WM_CLOSE, 0, 0);
            WINDOWS.lock().unwrap().remove(&hmonitor);
        }
        let name = CString::new("wwm_app_bar").expect("Failed to transform string to cstring");

        debug!("Unregistering window class");

        UnregisterClassA(
            name.as_ptr(),
            winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut()),
        );

        *FONT.lock().unwrap() = 0;
    }
}

#[allow(dead_code)]
pub fn hide() {
    unsafe {
        let hwnds: Vec<i32> = WINDOWS
            .lock()
            .unwrap()
            .iter()
            .map(|(_, hwnd)| *hwnd)
            .collect();
        for hwnd in hwnds {
            ShowWindow(hwnd as HWND, SW_HIDE);
        }
    }
}

pub fn show() {
    unsafe {
        let hwnds: Vec<i32> = WINDOWS
            .lock()
            .unwrap()
            .iter()
            .map(|(_, hwnd)| *hwnd)
            .collect();
        for hwnd in hwnds {
            ShowWindow(hwnd as HWND, SW_SHOW);
            draw_workspaces(hwnd as HWND);
            draw_datetime(hwnd as HWND).expect("Failed to draw datetime");
        }
    }
}

pub fn draw_datetime(hwnd: HWND) -> Result<(), util::WinApiResultError> {
    if !hwnd.is_null() {
        let mut rect = RECT::default();

        unsafe {
            util::winapi_nullable_to_result(GetClientRect(hwnd, &mut rect))?;
            let text = format!("{}", chrono::Local::now().format(&CONFIG.lock().unwrap().app_bar_time_pattern));
            let text_len = text.len() as i32;
            let c_text = CString::new(text).unwrap();
            let hmonitor = WINDOWS
                .lock()
                .unwrap()
                .iter()
                .find(|(_, v)| **v == hwnd as i32)
                .map(|(k, _)| *k)
                .expect("Failed to get current monitor");

            let display = get_display_by_hmonitor(hmonitor);

            let hdc = util::winapi_ptr_to_result(GetDC(hwnd))?;

            set_font(hdc);

            let mut size = SIZE::default();

            util::winapi_nullable_to_result(GetTextExtentPoint32A(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut size,
            ))?;

            rect.left = display.width() / 2 - (size.cx / 2) - 10;
            rect.right = display.width() / 2 + (size.cx / 2) + 10;

            //TODO: handle error
            if CONFIG.lock().unwrap().light_theme {
                SetTextColor(hdc, 0x00333333);
            } else {
                SetTextColor(hdc, 0x00ffffff);
            }

            SetBkColor(hdc, CONFIG.lock().unwrap().app_bar_bg as u32);

            util::winapi_nullable_to_result(DrawTextA(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            ))?;

            let text = format!("{}", chrono::Local::now().format(&CONFIG.lock().unwrap().app_bar_date_pattern));
            let text_len = text.len() as i32;
            let c_text = CString::new(text).unwrap();

            util::winapi_nullable_to_result(GetTextExtentPoint32A(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut size,
            ))?;

            rect.right = display.width() - 10;
            rect.left = rect.right - size.cx;

            util::winapi_nullable_to_result(DrawTextA(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            ))?;

            ReleaseDC(hwnd, hdc);
        }
    }

    Ok(())
}

pub fn draw_workspace(
    hwnd: HWND,
    idx: i32,
    id: i32,
    focused: bool,
) -> Result<(), util::WinApiResultError> {
    if !hwnd.is_null() {
        let mut rect = RECT::default();
        let height = *HEIGHT.lock().unwrap();

        unsafe {
            util::winapi_nullable_to_result(GetClientRect(hwnd, &mut rect))?;
            rect.left += height * idx;
            rect.right = rect.left + height;

            let hdc = util::winapi_ptr_to_result(GetDC(hwnd))?;

            set_font(hdc);

            let app_bar_bg = CONFIG.lock().unwrap().app_bar_bg;

            SetBkMode(hdc, TRANSPARENT as i32);

            if CONFIG.lock().unwrap().light_theme {
                SetTextColor(hdc, 0x00333333);

                let brush = if focused {
                    CreateSolidBrush(util::scale_color(app_bar_bg, 0.75) as u32)
                } else {
                    CreateSolidBrush(util::scale_color(app_bar_bg, 0.9) as u32)
                };

                FillRect(hdc, &rect, brush);
                DeleteObject(brush as *mut std::ffi::c_void);
            } else {
                SetTextColor(hdc, 0x00ffffff);

                let brush = if focused {
                    CreateSolidBrush(util::scale_color(app_bar_bg, 2.0) as u32)
                } else {
                    CreateSolidBrush(util::scale_color(app_bar_bg, 1.5) as u32)
                };

                FillRect(hdc, &rect, brush);
                DeleteObject(brush as *mut std::ffi::c_void);
            }

            let id_str = id.to_string();
            let len = id_str.len() as i32;
            let id_cstr = CString::new(id_str).unwrap();

            util::winapi_nullable_to_result(DrawTextA(
                hdc,
                id_cstr.as_ptr(),
                len,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            ))?;

            ReleaseDC(hwnd, hdc);
        }
    }

    Ok(())
}
