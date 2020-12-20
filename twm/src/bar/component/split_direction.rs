use crate::split_direction::SplitDirection;

use super::{Component, ComponentText};

pub fn create(vertical: String, horizontal: String) -> Component {
    Component::new("SplitDirection", move |ctx| {
        Ok(vec![ComponentText::new().with_display_text(
            ctx.state
                .get_display_by_id(ctx.display.id)
                .and_then(|d| d.get_focused_grid())
                .map(|w| match w.next_axis {
                    SplitDirection::Horizontal => horizontal.clone(),
                    SplitDirection::Vertical => vertical.clone(),
                })
                .unwrap_or("".into()),
        )])
    })
}
