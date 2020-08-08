use crate::workspace::{change_workspace, is_visible_workspace};
use crate::CONFIG;
use crate::{display::get_display_by_hmonitor, util, GRIDS};
use alignment::Alignment;
use component::{date::DateComponent, mode::ModeComponent, Component, time::TimeComponent};
use font::load_font;
use lazy_static::lazy_static;
use log::{debug, error, info};
use std::collections::HashMap;
use std::{ffi::CString, fmt::Debug, sync::Mutex};
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::shared::windef::SIZE;
use winapi::shared::windowsx::GET_X_LPARAM;
use winapi::um::wingdi::GetTextExtentPoint32A;
use winapi::um::wingdi::SetBkColor;
use winapi::um::wingdi::SetTextColor;
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
use winapi::um::winuser::{
    DrawTextA, GetDC, ReleaseDC, DT_CENTER, DT_END_ELLIPSIS, DT_SINGLELINE, DT_VCENTER,
    WM_SETCURSOR,
};

pub mod alignment;
pub mod close;
pub mod component;
pub mod create;
pub mod draw_datetime;
pub mod draw_mode;
pub mod draw_workspace;
pub mod draw_workspaces;
pub mod font;
pub mod redraw;
pub mod visibility;

lazy_static! {
    pub static ref HEIGHT: Mutex<i32> = Mutex::new(0);
    //HMONITOR, HWND
    pub static ref WINDOWS: Mutex<HashMap<i32, i32>> = Mutex::new(HashMap::new());
    static ref ITEMS: Mutex<Vec<Item>> = Mutex::new(Vec::new());
    pub static ref FONT: Mutex<i32> = Mutex::new(0);
    pub static ref REDRAW_REASON: Mutex<RedrawReason> = Mutex::new(RedrawReason::Time);
}

#[derive(Clone, Debug, PartialEq)]
pub enum RedrawReason {
    Time,
    Workspace,
    Mode(Option<String>),
    Initialize,
}

pub struct Item {
    pub left: i32,
    pub right: i32,
    pub alignment: Alignment,
    pub component: Box<dyn Component + Send>
}

impl Item {
    pub fn new(alignment: Alignment, component: Box<dyn Component + Send>) -> Self {
        Self {
            alignment,
            component,
            left: 0,
            right: 0
        }
    }
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
        let mut items = ITEMS.lock().unwrap();
        let reason = REDRAW_REASON.lock().unwrap().clone();
        let mut paint = PAINTSTRUCT::default();

        let hmonitor = WINDOWS
            .lock()
            .unwrap()
            .iter()
            .find(|(_, v)| **v == hwnd as i32)
            .map(|(k, _)| *k)
            .ok_or("Failed to get current monitor")
            .map_err(|e| error!("{}", e))
            .unwrap();

        let display = get_display_by_hmonitor(hmonitor);
        let padding = 20;
        let height = CONFIG.lock().unwrap().app_bar_height;
        let light_theme = CONFIG.lock().unwrap().light_theme;
        let app_bar_bg = CONFIG.lock().unwrap().app_bar_bg as u32;

        debug!("Redraw reason {:?}", reason);

        BeginPaint(hwnd, &mut paint);

        let mut left = display.left;
        let mut right = display.right;

        for item in items.iter_mut() {
            let mut rect = RECT::default();
            let component_text = item.component.render();
            let text = component_text.get_text();
            let text_len = text.len() as i32;
            let c_text = CString::new(text).unwrap();

            let hdc = util::winapi_ptr_to_result(GetDC(hwnd))
                .map_err(|e| error!("{}", e))
                .unwrap();

            font::set_font(hdc);

            // Calculate the width of the component
            //
            // The component can either specify a max width or return None.
            // When None is returned the component takes as much space as is needed to display the text
            let width = item.component.get_width().unwrap_or_else(|| {
                let mut size = SIZE::default();

                util::winapi_nullable_to_result(GetTextExtentPoint32A(
                    hdc,
                    c_text.as_ptr(),
                    text_len,
                    &mut size,
                ))
                .map_err(|e| error!("{}", e))
                .unwrap();

                size.cx
            });
            
            match item.alignment {
                Alignment::Left => {
                    rect.left = left;
                    rect.right = rect.left + width;

                    left = rect.right;
                    left += padding;
                }
                Alignment::Center => {}
                Alignment::Right => {
                    rect.right = right;
                    rect.left = rect.right - width;

                    right = rect.left;
                    right -= padding;
                }
            }


            rect.bottom = height;

            let fg = component_text.get_fg().unwrap_or(if light_theme {
                0x00333333
            } else {
                0x00ffffff
            });

            let bg = component_text.get_bg().unwrap_or(app_bar_bg);

            SetTextColor(hdc, fg);
            SetBkColor(hdc, bg);

            item.left = rect.left;
            item.right = rect.right;

            util::winapi_nullable_to_result(DrawTextA(
                hdc,
                c_text.as_ptr(),
                text_len,
                &mut rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            ))
            .map_err(|e| error!("{}", e))
            .unwrap();

            ReleaseDC(hwnd, hdc);
        }

        EndPaint(hwnd, &paint);
    }

    DefWindowProcA(hwnd, msg, w_param, l_param)
}

pub fn init() {
    let mut items = ITEMS.lock().unwrap();

    items.push(Item::new(Alignment::Left, Box::new(TimeComponent::default())));
    items.push(Item::new(Alignment::Left, Box::new(ModeComponent::default())));
    items.push(Item::new(Alignment::Right, Box::new(DateComponent::default())));

    create::create();
}
