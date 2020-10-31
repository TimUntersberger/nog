use super::{Component, ComponentText};
use chrono::Local;

pub fn create(pattern: String) -> Component {
    Component::new("Date", move |_| {
        let text = Local::now().format(&pattern).to_string();

        vec![ComponentText::new().with_display_text(text)]
    })
}
