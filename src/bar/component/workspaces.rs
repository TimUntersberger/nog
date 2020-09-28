use super::{Component, ComponentText};
use crate::{
    util,
};
use std::sync::Arc;

pub fn create() -> Component {
    Component::new(
        "Workspaces",
        Arc::new(|ctx| {
            let light_theme = ctx.state.config.light_theme;
            let workspace_settings = ctx.state.config.workspace_settings.clone();
            let bar_color = ctx.state.config.bar.color;

            // TODO: Extract this into a function of Display
            ctx.state.grids
                .iter()
                .filter(|g| {
                    (!g.tiles.is_empty() || ctx.state.is_workspace_visible(g.id))
                        && g.display.id == ctx.display.id
                })
                .map(|grid| {
                    let bg = if light_theme {
                        if ctx.state.workspace_id == grid.id {
                            util::scale_color(bar_color, 0.75) as u32
                        } else {
                            util::scale_color(bar_color, 0.9) as u32
                        }
                    } else {
                        if ctx.state.workspace_id == grid.id {
                            util::scale_color(bar_color, 2.0) as u32
                        } else {
                            util::scale_color(bar_color, 1.5) as u32
                        }
                    };

                    let mut text = format!(" {} ", grid.id.to_string());

                    if let Some(settings) = workspace_settings.iter().find(|s| s.id == grid.id) {
                        if !settings.text.is_empty() {
                            text = settings.text.clone();
                        }
                    }

                    ComponentText::Colored(None, Some(bg), text)
                })
                .collect()
        }),
    )
    .with_on_click(Arc::new(|_, display, idx| {
        let display = display.clone();

        //Note: have to run this in a new thread, because locking a mutex twice on a thread causes a
        //deadlock.
        std::thread::spawn(move || {
            let maybe_id = GRIDS
                .lock()
                .iter()
                .filter(|g| {
                    (!g.tiles.is_empty() || is_visible_workspace(g.id))
                        && g.display.id == display.id
                })
                .map(|g| g.id)
                .skip(idx)
                .next();

            if let Some(id) = maybe_id {
                let _ = change_workspace(id, true);
            }
        });
    }))
    .to_owned()
}
