use super::get_windows;

pub fn redraw() {
    for window in get_windows() {
        window.redraw();
    }
}
