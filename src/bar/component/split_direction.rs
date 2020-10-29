use crate::split_direction::SplitDirection;

use super::{Component, ComponentText};

pub fn create(vertical: String, horizontal: String) -> Component {
    Component::new(
        "SplitDirection",
        move |ctx| {
            vec![ComponentText::new().with_display_text(
                ctx.state
                    .get_display_by_id(ctx.display.id)
                    .and_then(|d| d.get_focused_grid())
                    .and_then(|g| g.get_focused_tile())
                    .map(|t| match t.split_direction {
                        SplitDirection::Horizontal => horizontal.clone(),
                        SplitDirection::Vertical => vertical.clone()
                    })
                    .unwrap_or("".into()),
            )]
        },
    )
}
