use super::{Component, ComponentText};
use std::sync::Arc;

pub fn create(amount: i32) -> Component {
    Component::new(
        "Padding",
        Arc::new(move |_| {
            vec![ComponentText::Basic(
                " ".to_string().repeat(amount as usize),
            )]
        }),
    )
}
