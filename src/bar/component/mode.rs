use super::{Component, ComponentText};
use crate::keybindings::MODE;

fn render(_: &Component, _: i32) -> Vec<ComponentText> {
    vec![ComponentText::Basic(
        MODE.lock()
            .unwrap()
            .clone()
            .map(|m| format!("{} is active", m))
            .unwrap_or_default(),
    )]
}

pub fn create() -> Component {
    Component::new(render)
}
