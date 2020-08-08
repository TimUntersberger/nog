use super::{Component, ComponentText};
use crate::CONFIG;
use chrono::Local;

fn render(_: &Component, _: i32) -> Vec<ComponentText> {
    let config = CONFIG.lock().unwrap();
    let text = Local::now()
        .format(&config.app_bar_time_pattern)
        .to_string();

    vec![ComponentText::Basic(text)]
}

pub fn create() -> Component {
    Component::new(render)
}
