use super::{Component, ComponentText};
use crate::split_direction::SplitDirection;
use crate::AppState;
use parking_lot::Mutex;
use std::sync::Arc;

pub fn create(state_arc: Arc<Mutex<AppState>>, vertical: String, horizontal: String) -> Component {
    Component::new("SplitDirection", move |display_id| {
        Ok(vec![ComponentText::new().with_display_text(
            state_arc
                .lock()
                .get_display_by_id(display_id)
                .and_then(|d| d.get_focused_grid())
                .map(|w| match w.next_axis {
                    SplitDirection::Horizontal => horizontal.clone(),
                    SplitDirection::Vertical => vertical.clone(),
                })
                .unwrap_or("".into()),
        )])
    })
}
