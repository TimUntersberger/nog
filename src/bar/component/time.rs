use super::{Component, ComponentText};
use crate::{display::Display, CONFIG};
use chrono::Local;
use std::sync::Arc;

fn render(_: &Component, _: &Display) -> Vec<ComponentText> {
    let config = CONFIG.lock().unwrap();
    let text = Local::now()
        .format(&config.app_bar_time_pattern)
        .to_string();

    vec![ComponentText::Basic(text)]
}

pub fn create() -> Component {
    Component::new("Time", Arc::new(render))
}
