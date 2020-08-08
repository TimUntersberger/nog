use super::{ComponentText, Component};
use crate::{bar::RedrawReason, CONFIG};
use chrono::Local;

#[derive(Default, Copy, Clone)]
pub struct DateComponent;

impl Component for DateComponent {
    fn get_width(&self) -> Option<i32> {
        None
    }
    fn render(&self) -> ComponentText {
        let config = CONFIG.lock().unwrap();
        let text = Local::now()
            .format(&config.app_bar_date_pattern)
            .to_string();

        ComponentText::Basic(text)
    }
    fn should_render(&self, reason: RedrawReason) -> bool {
       reason == RedrawReason::Time
    }
}