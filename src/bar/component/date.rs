use super::{Component, ComponentText, RenderFn};
use crate::{display::Display};
use chrono::Local;
use std::sync::Arc;

fn render(pattern: String) -> RenderFn {
    Arc::new(move |_: &Component, _: &Display| -> Vec<ComponentText> {
        let text = Local::now()
            .format(&pattern)
            .to_string();

        vec![ComponentText::Basic(text)]
    })
}

pub fn create(pattern: String) -> Component {
    Component::new("Date", render(pattern))
}
