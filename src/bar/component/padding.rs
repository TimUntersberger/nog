use super::{Component, ComponentText, RenderFn};
use crate::display::Display;
use std::sync::Arc;

fn render(amount: i32) -> RenderFn {
    Arc::new(move |_: &Component, _: &Display| {
        vec![ComponentText::Basic(" ".to_string().repeat(amount as usize))]
    })
}

pub fn create(amount: i32) -> Component {
    Component::new("Padding", render(amount))
}
