use crate::{display::Display, system::DisplayId, system::WindowId, window::Window};
use item::Item;
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

#[derive(Clone, Debug)]
pub struct Bar {
    window: Window,
    display_id: DisplayId,
    left: ItemSection,
    center: ItemSection,
    right: ItemSection,
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            window: Window::new(),
            display_id: DisplayId::default(),
            left: ItemSection::default(),
            center: ItemSection::default(),
            right: ItemSection::default(),
        }
    }
}

impl Bar {
    pub fn item_at_pos(&self, x: i32) -> Option<&Item> {
        for section in vec![&self.left, &self.center, &self.right] {
            if section.left <= x && x <= section.right {
                for item in section.items.iter() {
                    if item.left <= x && x <= item.right {
                        return Some(item);
                    }
                }
            }
        }

        None
    }
}
