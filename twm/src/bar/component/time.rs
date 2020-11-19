use super::{Component, ComponentText};
use chrono::Local;

pub fn create(pattern: String) -> Component {
    Component::new("Time", move |_| {
        vec![ComponentText::new().with_display_text(Local::now().format(&pattern).to_string())]
    })
}
