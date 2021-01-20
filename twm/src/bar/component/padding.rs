use super::{Component, ComponentText};

pub fn create(amount: i32) -> Component {
    Component::new("Padding", move |_| {
        Ok(vec![ComponentText::new().with_display_text(
            " ".to_string().repeat(amount as usize),
        )])
    })
}
