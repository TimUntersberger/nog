use crate::CONFIG;
use crate::{
    display::{get_display_by_hmonitor, Display},
    util,
    window::Window,
};
use font::load_font;
use lazy_static::lazy_static;
use log::info;

use std::sync::Mutex;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::HDC;

use winapi::shared::windef::HWND;
use winapi::shared::windef::POINT;
use winapi::shared::windef::RECT;

use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::wingdi::DeleteObject;

use winapi::um::wingdi::SetBkColor;
use winapi::um::wingdi::SetTextColor;
use winapi::um::winuser::BeginPaint;
use winapi::um::winuser::DefWindowProcA;
use winapi::um::winuser::EndPaint;

use component::{Component, ComponentText};
use item::Item;
use item_section::ItemSection;
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
    DrawTextW, FillRect, GetCursorPos, GetDC, ReleaseDC, DT_CALCRECT, DT_SINGLELINE, DT_VCENTER,
    IDC_HAND, WM_SETCURSOR,
};

pub mod close;
pub mod component;
pub mod create;
pub mod font;
pub mod item;
pub mod item_section;
pub mod redraw;
pub mod visibility;

lazy_static! {
    pub static ref BARS: Mutex<Vec<Bar>> = Mutex::new(Vec::new());
    pub static ref FONT: Mutex<i32> = Mutex::new(0);
}

#[derive(Clone)]
pub struct Bar {
    window: Window,
    hmonitor: i32,
    left: ItemSection,
    center: ItemSection,
    right: ItemSection,
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            window: Window::default(),
            hmonitor: 0,
            left: ItemSection::default(),
            center: ItemSection::default(),
            right: ItemSection::default(),
        }
    }
}

unsafe fn calculate_width(hdc: HDC, component_text: &ComponentText) -> i32 {
    let text = component_text.get_text();
    let c_text = util::to_widestring(&text);

    let mut rect = RECT::default();

    DrawTextW(
        hdc,
        c_text.as_ptr(),
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
        .unwrap_or(CONFIG.lock().unwrap().bar.color as u32);

    SetTextColor(hdc, fg);
    SetBkColor(hdc, bg);
    DrawTextW(hdc, c_text.as_ptr(), -1, rect, DT_VCENTER | DT_SINGLELINE);
}

unsafe fn draw_components(
    hdc: HDC,
    display: &Display,
    height: i32,
    mut offset: i32,
    components: &[Component],
) {
    for component in components {
        let component_texts = component.render(display);

        for (_i, component_text) in component_texts.iter().enumerate() {
            let width = calculate_width(hdc, &component_text);

            let mut rect = RECT::default();

            rect.left = offset;
            rect.right = rect.left + width;
            rect.bottom = height;

            offset = rect.right;

            draw_component_text(hdc, &mut rect, &component_text);
        }
    }
}

unsafe fn components_to_section(
    hdc: HDC,
    display: &Display,
    components: &[Component],
) -> ItemSection {
    let mut section = ItemSection::default();
    let mut component_offset = 0;

    for component in components {
        let mut item = Item::default();
        let mut component_text_offset = 0;
        let mut component_width = 0;

        for component_text in component.render(&display) {
            let width = calculate_width(hdc, &component_text);
            let left = component_text_offset;
            let right = component_text_offset + width;

            item.widths.push((left, right));

            component_width += width;
            component_text_offset += width;
        }

        item.left = component_offset;
        item.right = item.left + component_width;
        item.component = component.clone();

        section.items.push(item);

        component_offset += component_width;
    }

    section.right = component_offset;

    section
}

unsafe fn clear_section(hdc: HDC, height: i32, left: i32, right: i32) {
    let brush = CreateSolidBrush(CONFIG.lock().unwrap().bar.color as u32);
    let mut rect = RECT {
        left,
        right,
        top: 0,
        bottom: height,
    };

    FillRect(hdc, &mut rect, brush);

    DeleteObject(brush as *mut std::ffi::c_void);
}

fn update_bar(bar: Bar) {
    let mut bars = BARS.lock().unwrap();

    let mut_bar = bars
        .iter_mut()
        .find(|b| b.hmonitor == bar.hmonitor)
        .unwrap();

    *mut_bar = bar;
}

unsafe extern "system" fn window_cb(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_CLOSE {
        let mut bars = BARS.lock().unwrap();
        let idx = bars
            .iter()
            .position(|b| b.window.id == hwnd as i32)
            .unwrap();
        bars.remove(idx);
    } else if msg == WM_SETCURSOR {
        let mut point = POINT::default();
        GetCursorPos(&mut point);

        let bar = get_bar_by_hwnd(hwnd as i32).unwrap();
        let display = get_display_by_hmonitor(bar.hmonitor);
        let x = point.x - display.left;
        let mut found = false;

        for section in vec![bar.left, bar.center, bar.right] {
            if section.left <= x && x <= section.right {
                for item in section.items {
                    if item.left <= x && x <= item.right {
                        if item.component.is_clickable {
                            found = true;
                        }

                        break;
                    }
                }
            }
        }

        if found {
            SetCursor(LoadCursorA(std::ptr::null_mut(), IDC_HAND as *const i8));
        } else {
            SetCursor(LoadCursorA(std::ptr::null_mut(), IDC_ARROW as *const i8));
        }
    } else if msg == WM_DEVICECHANGE {
        println!("Device change!");
    } else if msg == WM_LBUTTONDOWN {
        let mut point = POINT::default();
        GetCursorPos(&mut point);

        let bar = get_bar_by_hwnd(hwnd as i32).unwrap();
        let display = get_display_by_hmonitor(bar.hmonitor);
        let x = point.x - display.left;

        for section in vec![bar.left, bar.center, bar.right] {
            if section.left <= x && x <= section.right {
                for item in section.items.iter() {
                    if item.left <= x && x <= item.right {
                        if item.component.is_clickable {
                            for (i, width) in item.widths.iter().enumerate() {
                                if width.0 <= x && x <= width.1 {
                                    item.component.on_click(&display, i);
                                }
                            }
                        }

                        break;
                    }
                }
            }
        }
    } else if msg == WM_CREATE {
        info!("loading font");
        load_font();
    } else if msg == WM_PAINT {
        let bar_config = CONFIG.lock().unwrap().bar.clone();
        let mut paint = PAINTSTRUCT::default();

        let mut bar = get_bar_by_hwnd(hwnd as i32).unwrap();
        let display = get_display_by_hmonitor(bar.hmonitor);

        BeginPaint(hwnd, &mut paint);

        let hdc = GetDC(hwnd);

        font::set_font(hdc);

        let left = components_to_section(hdc, &display, &bar_config.components.left);

        let mut center = components_to_section(hdc, &display, &bar_config.components.center);
        center.left = display.width() / 2 - center.right / 2;
        center.right += center.left;

        let mut right = components_to_section(hdc, &display, &bar_config.components.right);
        right.left = display.width() - right.right;
        right.right += right.left;

        draw_components(
            hdc,
            &display,
            bar_config.height,
            left.left,
            &bar_config.components.left,
        );
        draw_components(
            hdc,
            &display,
            bar_config.height,
            center.left,
            &bar_config.components.center,
        );
        draw_components(
            hdc,
            &display,
            bar_config.height,
            right.left,
            &bar_config.components.right,
        );

        if bar.left.width() > left.width() {
            clear_section(hdc, bar_config.height, left.right, bar.left.right);
        }

        if bar.center.width() > center.width() {
            let delta = (bar.center.right - center.right) / 2;
            clear_section(
                hdc,
                bar_config.height,
                bar.center.left,
                bar.center.left + delta,
            );
            clear_section(
                hdc,
                bar_config.height,
                bar.center.right - delta,
                bar.center.right,
            );
        }

        if bar.right.width() > right.width() {
            clear_section(hdc, bar_config.height, bar.right.left, right.left);
        }

        bar.left = left;
        bar.center = center;
        bar.right = right;

        update_bar(bar);

        ReleaseDC(hwnd, hdc);
        EndPaint(hwnd, &paint);
    }

    DefWindowProcA(hwnd, msg, w_param, l_param)
}

pub fn get_bar_by_hwnd(hwnd: i32) -> Option<Bar> {
    BARS.lock()
        .unwrap()
        .iter()
        .cloned()
        .find(|b| b.window.id == hwnd)
}

pub fn get_bar_by_hmonitor(hmonitor: i32) -> Option<Bar> {
    BARS.lock()
        .unwrap()
        .iter()
        .cloned()
        .find(|b| b.hmonitor == hmonitor)
}

pub fn get_windows() -> Vec<Window> {
    BARS.lock()
        .unwrap()
        .iter()
        .map(|bar| &bar.window)
        .cloned()
        .collect()
}
