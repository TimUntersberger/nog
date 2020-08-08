use crate::workspace::{change_workspace, is_visible_workspace};
use crate::CONFIG;
use crate::{display::get_display_by_hmonitor, util, GRIDS};
use alignment::Alignment;
use component::{date, mode, time, workspaces, Component, ComponentText};
use font::load_font;
use lazy_static::lazy_static;
use log::{debug, error, info};
use std::collections::HashMap;
use std::{cmp, ffi::CString, fmt::Debug, sync::Mutex};
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HDC;
use winapi::shared::windef::HMONITOR;
use winapi::shared::windef::HWND;
use winapi::shared::windef::POINT;
use winapi::shared::windef::RECT;
use winapi::shared::windef::SIZE;
use winapi::shared::windowsx::GET_X_LPARAM;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::wingdi::DeleteObject;
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
    DrawTextA, FillRect, GetCursorPos, GetDC, ReleaseDC, DT_CENTER, DT_END_ELLIPSIS, DT_SINGLELINE,
    DT_VCENTER, IDC_HAND, WM_SETCURSOR,
};

pub mod alignment;
pub mod close;
pub mod component;
pub mod create;
pub mod font;
pub mod redraw;
pub mod visibility;

lazy_static! {
    pub static ref HEIGHT: Mutex<i32> = Mutex::new(0);
    //HMONITOR, HWND
    pub static ref WINDOWS: Mutex<HashMap<i32, i32>> = Mutex::new(HashMap::new());
    //TODO: change to RwLock
    static ref ITEMS: Mutex<Vec<Item>> = Mutex::new(Vec::new());
    pub static ref FONT: Mutex<i32> = Mutex::new(0);
}

pub struct Item {
    pub left: i32,
    pub right: i32,
    pub alignment: Alignment,
    pub component: Component,
}

impl Item {
    pub fn new(alignment: Alignment, component: Component) -> Self {
        Self {
            alignment,
            component,
            left: 0,
            right: 0,
        }
    }
}

unsafe fn calculate_width(hdc: HDC, str: &CString, len: i32, item: &Item) -> i32 {
    cmp::max(
        {
            let mut size = SIZE::default();

            util::winapi_nullable_to_result(GetTextExtentPoint32A(
                hdc,
                str.as_ptr(),
                len,
                &mut size,
            ))
            .map_err(|e| error!("{}", e))
            .unwrap();

            size.cx
        },
        item.right - item.left,
    )
}

unsafe fn draw_component_text(hdc: HDC, rect: &mut RECT, component_text: &ComponentText) {
    let text = component_text.get_text();
    let text_len = text.len();
    let c_text = CString::new(text).unwrap();

    let fg = component_text
        .get_fg()
        .unwrap_or(if CONFIG.lock().unwrap().light_theme {
            0x00333333
        } else {
            0x00ffffff
        });

    let bg = component_text
        .get_bg()
        .unwrap_or(CONFIG.lock().unwrap().app_bar_bg as u32);

    if text_len == 0 {
        let brush = CreateSolidBrush(bg);

        FillRect(hdc, rect, brush);

        DeleteObject(brush as *mut std::ffi::c_void);
    } else {
        SetTextColor(hdc, fg);
        SetBkColor(hdc, bg);

        DrawTextA(
            hdc,
            c_text.as_ptr(),
            text_len as i32,
            rect,
            DT_VCENTER | DT_SINGLELINE,
        );
    }
}

fn calculate_rect(item: &Item, left: &mut i32, right: &mut i32, width: i32, height: i32) -> RECT {
    let mut rect = RECT::default();

    match item.alignment {
        Alignment::Left => {
            rect.left = *left;
            rect.right = rect.left + width;

            *left = rect.right;
        }
        Alignment::Center => {}
        Alignment::Right => {
            rect.right = *right;
            rect.left = rect.right - width;

            *right = rect.left;
        }
    }

    rect.bottom = height;

    rect
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
        let mut point = POINT::default();
        GetCursorPos(&mut point);

        let items = ITEMS.lock().unwrap();
        let maybe_item = items.iter().find(|item| {
            item.component.is_clickable && item.left <= point.x && point.x <= item.right
        });

        if maybe_item.is_some() {
            SetCursor(LoadCursorA(std::ptr::null_mut(), IDC_HAND as *const i8));
        } else {
            SetCursor(LoadCursorA(std::ptr::null_mut(), IDC_ARROW as *const i8));
        }
    } else if msg == WM_DEVICECHANGE {
        println!("Device change!");
    } else if msg == WM_LBUTTONDOWN {
        let mut point = POINT::default();
        GetCursorPos(&mut point);

        let items = ITEMS.lock().unwrap();
        let maybe_item = items.iter().find(|item| {
            item.component.is_clickable && item.left <= point.x && point.x <= item.right
        });
        if let Some(item) = maybe_item {
            item.component.on_click();
        }
    } else if msg == WM_CREATE {
        info!("loading font");
        load_font();
    } else if msg == WM_PAINT {
        let mut items = ITEMS.lock().unwrap();
        let mut paint = PAINTSTRUCT::default();

        let hmonitor = get_monitor_by_hwnd(hwnd as i32);
        let display = get_display_by_hmonitor(hmonitor);
        let height = CONFIG.lock().unwrap().app_bar_height;

        BeginPaint(hwnd, &mut paint);

        let mut left = display.left;
        let mut right = display.right;

        for item in items.iter_mut() {
            let hdc = util::winapi_ptr_to_result(GetDC(hwnd))
                .map_err(|e| error!("{}", e))
                .unwrap();

            font::set_font(hdc);
            let component_texts = item.component.render(hmonitor);
            let mut component_texts_iter = component_texts.iter();

            if let Some(component_text) = component_texts_iter.next() {
                let text = component_text.get_text();
                let text_len = text.len() as i32;
                let c_text = CString::new(text).unwrap();

                let width = calculate_width(hdc, &c_text, text_len, item);
                let mut rect = calculate_rect(item, &mut left, &mut right, width, height);

                item.left = rect.left;
                item.right = rect.right;

                draw_component_text(hdc, &mut rect, &component_text);
            }

            for component_text in component_texts_iter {
                let text = component_text.get_text();
                let text_len = text.len() as i32;
                let c_text = CString::new(text).unwrap();

                let width = calculate_width(hdc, &c_text, text_len, item);
                let mut rect = calculate_rect(item, &mut left, &mut right, width, height);

                match item.alignment {
                    Alignment::Left => item.right = rect.right,
                    Alignment::Right => item.left = rect.left,
                    Alignment::Center => {}
                };

                draw_component_text(hdc, &mut rect, &component_text);
            }

            ReleaseDC(hwnd, hdc);
        }

        EndPaint(hwnd, &paint);
    }

    DefWindowProcA(hwnd, msg, w_param, l_param)
}

pub fn init() {
    let mut items = ITEMS.lock().unwrap();

    items.push(Item::new(Alignment::Left, time::create()));
    items.push(Item::new(Alignment::Left, mode::create()));
    items.push(Item::new(Alignment::Right, date::create()));
    items.push(Item::new(Alignment::Right, workspaces::create()));

    create::create();
}

pub fn get_windows() -> Vec<i32> {
    WINDOWS
        .lock()
        .unwrap()
        .iter()
        .map(|(_, hwnd)| *hwnd)
        .collect::<Vec<i32>>()
}

fn get_monitor_by_hwnd(hwnd: i32) -> i32 {
    WINDOWS
        .lock()
        .unwrap()
        .iter()
        .find(|(_, v)| **v == hwnd)
        .map(|(k, _)| *k)
        .ok_or(format!("Failed to get monitor for hwnd {}", hwnd))
        .map_err(|e| error!("{}", e))
        .unwrap()
}
