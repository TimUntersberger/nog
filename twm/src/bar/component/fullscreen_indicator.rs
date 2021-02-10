use super::{AppState, Component, ComponentText};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

pub fn create(state_arc: Arc<Mutex<AppState>>, indicator: String) -> Component {
    Component::new("FullscreenIndicator", move |display_id| {
        Ok(vec![ComponentText::new().with_display_text(
            if let Some(state) = state_arc.try_lock_for(Duration::from_millis(super::LOCK_TIMEOUT)) {
                state.get_display_by_id(display_id)
                     .and_then(|d| d.get_focused_grid())
                     .map(|g| {
                         if g.is_fullscreened() {
                             indicator.clone()
                         } else {
                             "".into()
                         }
                     })
                     .unwrap_or("".into())
            } else {
                "".into()
            }
        )])
    })
}
