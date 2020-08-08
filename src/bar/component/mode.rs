use super::{ComponentText, Component};
use crate::{bar::RedrawReason, CONFIG};

#[derive(Default, Copy, Clone)]
pub struct ModeComponent;

impl Component for ModeComponent {
    fn get_width(&self) -> Option<i32> {
        None
    }
    fn render(&self) -> ComponentText {
        ComponentText::Basic("mode is active".into())
    }
    fn should_render(&self, reason: RedrawReason) -> bool {
       match reason {
           RedrawReason::Mode(_) => true,
           _ => false
       }
    }
}