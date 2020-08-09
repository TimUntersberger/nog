use crate::CONFIG;
use crate::{display::get_display_by_hmonitor, util};
use alignment::Alignment;
use component::{date, mode, time, workspaces, Component, ComponentText, padding};
use font::load_font;
use lazy_static::lazy_static;
use log::{error, info};
use std::collections::HashMap;
use std::{cmp, ffi::CString, sync::Mutex};
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HDC;

use winapi::shared::windef::HWND;
use winapi::shared::windef::POINT;
use winapi::shared::windef::RECT;
use winapi::shared::windef::SIZE;

use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::wingdi::DeleteObject;
use winapi::um::wingdi::GetTextExtentPoint32W;
use winapi::um::wingdi::GetTextExtentPoint32A;
use winapi::um::wingdi::SetBkColor;
use winapi::um::wingdi::SetTextColor;
use winapi::um::winuser::BeginPaint;
use winapi::um::winuser::DefWindowProcA;
use winapi::um::winuser::EndPaint;

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
    FillRect, GetCursorPos, GetDC, ReleaseDC, DT_SINGLELINE, DT_VCENTER, IDC_HAND,
    WM_SETCURSOR, DrawTextW, DT_CALCRECT,
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
    /// left, right, width
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

unsafe fn calculate_width(hdc: HDC, str: &Vec<u16>) -> i32 {
    let mut rect = RECT::default();

    DrawTextW(
        hdc,
        str.as_ptr(),
        -1,
        &mut rect,
        DT_VCENTER | DT_SINGLELINE | DT_CALCRECT,
    );

    (rect.right - rect.left).abs()
}

unsafe fn draw_component_text(hdc: HDC, rect: &mut RECT, component_text: &ComponentText) {
    let text = component_text.get_text();

    if text.is_empty() {
        return;
    }

    let c_text = util::to_widestring(&text);

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

    SetTextColor(hdc, fg);
    SetBkColor(hdc, bg);
    DrawTextW(
        hdc,
        c_text.as_ptr(),
        -1,
        rect,
        DT_VCENTER | DT_SINGLELINE,
    );
}

fn calculate_rect(item: &Item, left: &mut i32, right: &mut i32, width: i32, height: i32) -> RECT {
    let mut rect = RECT::default();

    match item.alignment {
        Alignment::Left => {
            rect.left = *left;
            rect.right = rect.left + width;

            *left = rect.right;
        }
        Alignment::Center => {
            rect.left = *left;
            rect.right = rect.left + width;

            *left = rect.right;
        }
        Alignment::Right => {
            rect.right = *right;
            rect.left = rect.right - width;

            *right = rect.left;
        }
    }

    rect.bottom = height;

    rect
}

unsafe fn clear_section(hdc: HDC, height: i32, left: i32, right: i32) {
    let brush = CreateSolidBrush(CONFIG.lock().unwrap().app_bar_bg as u32);
    let mut rect = RECT {
        left,
        right,
        top: 0,
        bottom: height
    };

    FillRect(hdc, &mut rect, brush);

    DeleteObject(brush as *mut std::ffi::c_void);
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
        // left, center, right
        let mut prev_widths = (0, 0, 0);
        // left, center, right
        let mut widths = (0, 0, 0);
        let mut paint = PAINTSTRUCT::default();

        let hmonitor = get_monitor_by_hwnd(hwnd as i32);
        let display = get_display_by_hmonitor(hmonitor);
        let height = CONFIG.lock().unwrap().app_bar_height;

        BeginPaint(hwnd, &mut paint);

        let hdc = util::winapi_ptr_to_result(GetDC(hwnd))
            .map_err(|e| error!("{}", e))
            .unwrap();

        font::set_font(hdc);

        // indices
        let mut left = 0;
        let mut right = display.right - display.left;

        let mut center_item_indices: Vec<usize> = Vec::new();

        for (i, item) in items.iter_mut().enumerate() {
            match item.alignment {
                Alignment::Left => prev_widths.0 += (item.right - item.left).abs(),
                Alignment::Center => prev_widths.1 += (item.right - item.left).abs(),
                Alignment::Right => prev_widths.2 += (item.right - item.left).abs()
            };

            if item.alignment == Alignment::Center {
                center_item_indices.push(i);
            }

            let component_texts = item.component.render(&display);
            let mut component_texts_iter = component_texts.iter();

            if let Some(component_text) = component_texts_iter.next() {
                let text = component_text.get_text();
                let c_text = util::to_widestring(&text);

                let width = calculate_width(hdc, &c_text);

                match item.alignment {
                    Alignment::Left => widths.0 += width,
                    Alignment::Center => widths.1 += width,
                    Alignment::Right => widths.2 += width
                };

                if item.alignment != Alignment::Center {
                    let mut rect = calculate_rect(item, &mut left, &mut right, width, height);

                    item.left = rect.left;
                    item.right = rect.right;

                    draw_component_text(hdc, &mut rect, &component_text);
                }
            }

            for component_text in component_texts_iter {
                let text = component_text.get_text();
                let c_text = util::to_widestring(&text);

                let width = calculate_width(hdc, &c_text);

                match item.alignment {
                    Alignment::Left => widths.0 += width,
                    Alignment::Center => widths.1 += width,
                    Alignment::Right => widths.2 += width
                };

                if item.alignment != Alignment::Center {
                    let mut rect = calculate_rect(item, &mut left, &mut right, width, height);

                    match item.alignment {
                        Alignment::Left => item.right = rect.right,
                        Alignment::Right => item.left = rect.left,
                        Alignment::Center => {}
                    };

                    draw_component_text(hdc, &mut rect, &component_text);
                }
            }

        }

        println!("{} {} {}", prev_widths.0, prev_widths.1, prev_widths.2);
        println!("{} {} {}", widths.0, widths.1, widths.2);

        left = display.width() / 2 - widths.1 / 2;

        //draw center items
        for idx in center_item_indices {
            let mut item = items.get_mut(idx).expect("Failed to get item");
            let component_texts = item.component.render(&display);
            let mut component_texts_iter = component_texts.iter();

            if let Some(component_text) = component_texts_iter.next() {
                let text = component_text.get_text();
                let c_text = util::to_widestring(&text);

                let width = calculate_width(hdc, &c_text);
                let mut rect = calculate_rect(item, &mut left, &mut right, width, height);

                item.left = rect.left;
                item.right = rect.right;

                draw_component_text(hdc, &mut rect, &component_text);
            }

            for component_text in component_texts_iter {
                let text = component_text.get_text();
                let c_text = util::to_widestring(&text);

                let width = calculate_width(hdc, &c_text);
                let mut rect = calculate_rect(item, &mut left, &mut right, width, height);

                draw_component_text(hdc, &mut rect, &component_text);
            }

        }

        //cleanup previous render if there is any dangling rendered text
        if prev_widths.0 > widths.0 {
            clear_section(hdc, height, widths.0, prev_widths.0);
        }

        if prev_widths.1 > widths.1 {
            let prev_center_left = display.width() / 2 - prev_widths.1 / 2;
            let prev_center_right = prev_center_left + prev_widths.1;
            let delta = (prev_widths.1 - widths.1) / 2;
            clear_section(hdc, height, prev_center_left, prev_center_left + delta);
            clear_section(hdc, height, prev_center_right - delta, prev_center_right);
        }

        if prev_widths.2 > widths.2 {
            clear_section(hdc, height, display.width() - prev_widths.2, display.width() - widths.2);
        }

        ReleaseDC(hwnd, hdc);
        EndPaint(hwnd, &paint);
    }

    DefWindowProcA(hwnd, msg, w_param, l_param)
}

pub fn init() {
    let mut items = ITEMS.lock().unwrap();

    items.push(Item::new(Alignment::Left, workspaces::create()));
    // items.push(Item::new(Alignment::Center, time::create()));
    // items.push(Item::new(Alignment::Right, date::create()));
    // items.push(Item::new(Alignment::Right, padding::create(5)));
    // items.push(Item::new(Alignment::Right, mode::create()));
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
