use super::{Component, ComponentText};
use crate::{display::Display, keybindings::MODE};
use std::sync::Arc;

fn render(_: &Component, _: &Display) -> Vec<ComponentText> {
    vec![ComponentText::Basic(
        MODE.lock()
            .unwrap()
            .clone()
            .map(|m| format!("{} is active", m))
            .unwrap_or_default(),
    )]
}

pub fn create() -> Component {
    Component::new(Arc::new(render))
}
