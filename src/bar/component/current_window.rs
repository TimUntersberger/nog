use super::{Component, ComponentText};
use std::sync::Arc;

pub fn create() -> Component {
    Component::new(
        "ActiveMode",
        Arc::new(|ctx| {
            vec![ctx
                .state
                .get_current_grid()
                .get_focused_tile()
                .map(|t| ComponentText::Basic(t.window.title.clone()))
                .unwrap_or(ComponentText::Basic("".into()))]
        }),
    )
}
