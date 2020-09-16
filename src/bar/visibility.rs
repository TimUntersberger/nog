use super::{get_windows, get_bar_by_hwnd, redraw::redraw};

#[allow(dead_code)]
pub fn hide() {
    for window in get_windows() {
        if !window.is_hidden() {
            window.hide();
        }
    }
}

pub fn show() {
    for window in get_windows() {
        if window.is_hidden() {
            window.show();
        }
    }

    redraw();
}

pub fn toggle_by_hwnd(hwnd: i32) {
    let bar = get_bar_by_hwnd(hwnd).unwrap();
    match bar.window.is_hidden() {
        false => bar.window.hide(),
        true => {
            bar.window.show();
            redraw();
        }
    }
}
