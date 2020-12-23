use super::{AppState, Component, ComponentText};
use parking_lot::Mutex;
use std::sync::Arc;

pub fn create(state_arc: Arc<Mutex<AppState>>) -> Component {
    Component::new("ActiveMode", move |_| {
        Ok(vec![ComponentText::new().with_display_text(
            state_arc
                .lock()
                .keybindings_manager
                .get_mode()
                .map(|m| format!("{} is active", m))
                .unwrap_or_default(),
        )])
    })
}
