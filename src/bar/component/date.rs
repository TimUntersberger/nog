use super::{Component, ComponentText};
use chrono::Local;
use std::sync::Arc;

pub fn create(pattern: String) -> Component {
    Component::new(
        "Date",
        Arc::new(move |_| {
            let text = Local::now().format(&pattern).to_string();

            vec![ComponentText::Basic(text)]
        }),
    )
}
