use crate::{display::Display, system::WindowId, window::Window};
use item_section::ItemSection;
use parking_lot::Mutex;

pub mod close;
pub mod component;
pub mod create;
pub mod font;
pub mod item;
pub mod item_section;
pub mod redraw;
pub mod visibility;

pub static BARS: Mutex<Vec<Bar>> = Mutex::new(Vec::new());

#[derive(Clone, Debug)]
pub struct Bar {
    window: Window,
    display: Display,
    left: ItemSection,
    center: ItemSection,
    right: ItemSection,
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            window: Window::new(),
            display: Display::default(),
            left: ItemSection::default(),
            center: ItemSection::default(),
            right: ItemSection::default(),
        }
    }
}

pub fn get_bar_by_win_id(id: WindowId) -> Option<Bar> {
    with_bar_by(|b| b.window.id == id, |b| b.cloned())
}

pub fn with_bar_by<TF, TCb, TReturn>(f: TF, cb: TCb) -> TReturn
where
    TF: Fn(&&mut Bar) -> bool,
    TCb: Fn(Option<&mut Bar>) -> TReturn,
{
    cb(BARS.lock().iter_mut().find(f))
}

pub fn get_windows() -> Vec<Window> {
    BARS.lock()
        .clone()
        .iter()
        .map(|bar| bar.window.clone())
        .collect()
}
