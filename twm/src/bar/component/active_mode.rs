use super::{AppState, Component, ComponentText};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

pub fn create(state_arc: Arc<Mutex<AppState>>) -> Component {
    Component::new("ActiveMode", move |_| {
        Ok(vec![ComponentText::new().with_display_text(
            if let Some(state) = state_arc.try_lock_for(Duration::from_millis(super::LOCK_TIMEOUT))
            {
                state
                    .keybindings_manager
                    .try_get_mode()
                    .map(|m| match m {
                        Some(m) => format!("{} is active", m),
                        _ => "".into(),
                    })
                    .unwrap_or_default()
            } else {
                "".into()
            },
        )])
    })
}
