use super::{Component, ComponentText};
use crate::{display::Display, with_current_grid};
use std::sync::Arc;

fn render(_: &Component, _: &Display) -> Vec<ComponentText> {
    with_current_grid(|grid| {
        vec![grid
            .get_focused_tile()
            .map(|t| ComponentText::Basic(t.window.title.clone()))
            .unwrap_or(ComponentText::Basic("".into()))]
    })
}

pub fn create() -> Component {
    Component::new("ActiveMode", Arc::new(render))
}
