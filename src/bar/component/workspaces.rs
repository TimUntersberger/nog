use super::{Component, ComponentText};
use crate::{util, Event};

pub fn create() -> Component {
    Component::new("Workspaces", |ctx| {
        let light_theme = ctx.state.config.light_theme;
        let workspace_settings = ctx.state.config.workspace_settings.clone();
        let bar_color = ctx.state.config.bar.color;

        ctx.display
            .get_active_grids()
            .iter()
            .map(|grid| {
                let factor = if light_theme {
                    if ctx.state.workspace_id == grid.id {
                        0.75
                    } else {
                        0.9
                    }
                } else {
                    if ctx.state.workspace_id == grid.id {
                        2.0
                    } else {
                        1.5
                    }
                };
                ComponentText::new()
                    .with_display_text(
                        workspace_settings
                            .iter()
                            .find(|s| s.id == grid.id)
                            .map(|s| s.text.clone())
                            .filter(|t| !t.is_empty())
                            .unwrap_or(format!(" {} ", grid.id.to_string())),
                    )
                    .with_value(grid.id)
                    .with_background_color(util::scale_color(bar_color, factor))
            })
            .collect()
    })
    .with_on_click(|ctx| {
        let id = *ctx.value.downcast_ref::<i32>().unwrap();
        ctx.state
            .event_channel
            .sender
            .clone()
            .send(Event::ChangeWorkspace(id, true));
    })
    .to_owned()
}
