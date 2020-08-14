use super::{get_windows, redraw::redraw};

#[allow(dead_code)]
pub fn hide() {
    unsafe {
        for window in get_windows() {
            window.hide();
        }
    }
}

pub fn show() {
    unsafe {
        for window in get_windows() {
            window.show();
        }

        redraw();
    }
}
