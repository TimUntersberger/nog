use super::{Component, ComponentText};
use std::sync::Arc;

pub fn create() -> Component {
    Component::new(
        "CurrentWindow",
        Arc::new(|ctx| {
            vec![ctx
                .state
                .get_display_by_id(ctx.display.id)
                .and_then(|d| d.get_focused_grid())
                .and_then(|g| g.get_focused_tile())
                .map(|t| ComponentText::Basic(t.window.title.clone()))
                .unwrap_or(ComponentText::Basic("".into()))]
        }),
    )
}
