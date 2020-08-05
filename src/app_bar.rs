use crate::workspace::{change_workspace, is_visible_workspace};
use crate::CONFIG;
use crate::GRIDS;
use font::load_font;
use lazy_static::lazy_static;
use log::{debug, error, info};
use std::collections::HashMap;
use std::sync::Mutex;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HWND;
use winapi::shared::windowsx::GET_X_LPARAM;
use winapi::um::winuser::BeginPaint;
use winapi::um::winuser::DefWindowProcA;
use winapi::um::winuser::EndPaint;
use winapi::um::winuser::GetClientRect;
use winapi::um::winuser::LoadCursorA;
use winapi::um::winuser::SetCursor;
use winapi::um::winuser::IDC_ARROW;
use winapi::um::winuser::PAINTSTRUCT;
use winapi::um::winuser::WM_CLOSE;
use winapi::um::winuser::WM_CREATE;
use winapi::um::winuser::WM_DEVICECHANGE;
use winapi::um::winuser::WM_LBUTTONDOWN;
use winapi::um::winuser::WM_PAINT;
use winapi::um::winuser::WM_SETCURSOR;

pub mod close;
pub mod create;
pub mod draw_datetime;
pub mod draw_workspace;
pub mod draw_workspaces;
pub mod font;
pub mod redraw;
pub mod visibility;

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
    } else if msg == WM_DEVICECHANGE {
        println!("Device change!");
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
                if draw_datetime::draw_datetime(hwnd).is_err() {
                    error!("Failed to draw datetime");
                }
            }
            RedrawAppBarReason::Workspace => {
                debug!("Received redraw-app-bar event");
                draw_workspaces::draw_workspaces(hwnd);
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
