use super::{get_windows, redraw::redraw};

#[allow(dead_code)]
pub fn hide() {
    for window in get_windows() {
        window.hide();
    }
}

pub fn show() {
    for window in get_windows() {
        window.show();
    }

    redraw();
}
