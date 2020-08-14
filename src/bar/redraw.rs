use super::get_windows;

pub fn redraw() {
    unsafe {
        for window in get_windows() {
            window.redraw();
        }
    }
}
