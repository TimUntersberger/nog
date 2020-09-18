use super::{Component, ComponentText};
use crate::{
    display::Display,
    util,
    workspace::{change_workspace, is_visible_workspace},
    CONFIG, GRIDS, WORKSPACE_ID,
};
use std::sync::Arc;

fn render(_: &Component, display: &Display) -> Vec<ComponentText> {
    let light_theme = CONFIG.lock().light_theme;
    let workspace_settings = CONFIG.lock().workspace_settings.clone();
    let bar_color = CONFIG.lock().bar.color;
    let workspace_id = *WORKSPACE_ID.lock();

    GRIDS
        .lock()
        .iter()
        .filter(|g| {
            (!g.tiles.is_empty() || is_visible_workspace(g.id))
                && g.display.hmonitor == display.hmonitor
        })
        .map(|grid| {
            let bg = if light_theme {
                if workspace_id == grid.id {
                    util::scale_color(bar_color, 0.75) as u32
                } else {
                    util::scale_color(bar_color, 0.9) as u32
                }
            } else {
                if workspace_id == grid.id {
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
}

fn on_click(_: &Component, display: &Display, idx: usize) {
    let display = display.clone();

    //Note: have to run this in a new thread, because locking a mutex twice on a thread causes a
    //deadlock.
    std::thread::spawn(move || {
        let maybe_id = GRIDS
            .lock()
            .iter()
            .filter(|g| {
                (!g.tiles.is_empty() || is_visible_workspace(g.id))
                    && g.display.hmonitor == display.hmonitor
            })
            .map(|g| g.id)
            .skip(idx)
            .next();

        if let Some(id) = maybe_id {
            let _ = change_workspace(id, true);
        }
    });
}

pub fn create() -> Component {
    Component::new("Workspaces", Arc::new(render))
        .with_on_click(Arc::new(on_click))
        .to_owned()
}
