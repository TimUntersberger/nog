use super::get_windows;
use log::info;

pub fn close() {
    info!("Closing appbar");

    for window in get_windows() {
        window.close();
    }
}
