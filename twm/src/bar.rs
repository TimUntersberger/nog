use std::sync::Arc;

use crate::{system::DisplayId, window::Window, AppState};
use item::Item;
use crate::{SystemResult, SystemError};
use item_section::ItemSection;
use parking_lot::Mutex;

pub mod component;
pub mod create;
pub mod item;
pub mod item_section;

#[derive(Clone, Debug)]
pub struct Bar {
    pub window: Window,
    pub display_id: DisplayId,
    pub left: ItemSection,
    pub center: ItemSection,
    pub right: ItemSection,
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
    
    pub fn change_height(&self, height: i32) -> SystemResult {
        let nwin = self.window.get_native_window();
        let mut rect = nwin.get_rect()?;

        if rect.top == 0 {
            rect.bottom = height;
        } else {
            rect.top = rect.bottom - height;
        }

        nwin.set_window_pos(rect, None, None).map_err(|e| SystemError::Unknown(e))
    }
}

pub fn close_all(state_arc: Arc<Mutex<AppState>>) {
    let mut windows = Vec::new();

    for d in state_arc.lock().displays.iter_mut() {
        if let Some(b) = d.appbar.as_ref() {
            windows.push(b.window.clone())
        }
        d.appbar = None;
    }

    for w in windows {
        w.close();
    }
}
