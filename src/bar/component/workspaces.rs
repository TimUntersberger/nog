use super::{Component, ComponentText};
use crate::{
    display::Display,
    util,
    workspace::{change_workspace, is_visible_workspace},
    CONFIG, GRIDS, WORKSPACE_ID,
};
use std::sync::Arc;

fn render(_: &Component, display: &Display) -> Vec<ComponentText> {
    let light_theme = CONFIG.lock().unwrap().light_theme;
    let workspace_settings = CONFIG.lock().unwrap().workspace_settings.clone();
    let app_bar_bg = CONFIG.lock().unwrap().app_bar_bg;
    let workspace_id = *WORKSPACE_ID.lock().unwrap();

    GRIDS
        .lock()
        .unwrap()
        .iter()
        .filter(|g| {
            (!g.tiles.is_empty() || is_visible_workspace(g.id))
                && g.display.hmonitor == display.hmonitor
        })
        .map(|grid| {
            let (fg, bg) = if light_theme {
                let fg = 0x00333333;

                let bg = if workspace_id == grid.id {
                    util::scale_color(app_bar_bg, 0.75) as u32
                } else {
                    util::scale_color(app_bar_bg, 0.9) as u32
                };

                (fg, bg)
            } else {
                let fg = 0x00ffffff;
                let bg = if workspace_id == grid.id {
                    util::scale_color(app_bar_bg, 2.0) as u32
                } else {
                    util::scale_color(app_bar_bg, 1.5) as u32
                };

                (fg, bg)
            };
            ComponentText::Colored(
                fg,
                bg,
                workspace_settings
                    .iter()
                    .find(|s| s.id == grid.id)
                    .map(|g| g.text.clone())
                    .unwrap_or(format!(" {} ", grid.id.to_string())),
            )
        })
        .collect()
}

fn on_click(_: &Component, display: &Display, idx: usize) {
    let maybe_id = GRIDS
        .lock()
        .unwrap()
        .iter()
        .filter(|g| {
            (!g.tiles.is_empty() || is_visible_workspace(g.id))
                && g.display.hmonitor == display.hmonitor
        })
        .map(|g| g.id)
        .skip(idx)
        .next();

    if let Some(id) = maybe_id {
        let _ = change_workspace(id);
    }
}

pub fn create() -> Component {
    Component::new("Workspaces", Arc::new(render))
        .with_on_click(Arc::new(on_click))
        .to_owned()
}
