use super::{Component, ComponentText};
use crate::{util, AppState, Event};
use parking_lot::Mutex;
use std::sync::Arc;

pub fn create(state_arc: Arc<Mutex<AppState>>) -> Component {
    let state_arc2 = state_arc.clone();
    Component::new("Workspaces", move |display_id| {
        let state = state_arc.lock();
        let light_theme = state.config.light_theme;
        let workspace_settings = state.config.workspace_settings.clone();
        let bar_color = state.config.bar.color;

        let mut grids = state
            .get_display_by_id(display_id)
            .unwrap()
            .get_active_grids();
        grids.sort_by_key(|g| g.id);

        Ok(grids
            .iter()
            .map(|grid| {
                let factor = if light_theme {
                    if state.workspace_id == grid.id {
                        0.75
                    } else {
                        0.9
                    }
                } else {
                    if state.workspace_id == grid.id {
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
            .collect())
    })
    .with_on_click(move |_, value, _| {
        let id = *value.downcast_ref::<i32>().unwrap();
        state_arc2
            .lock()
            .event_channel
            .sender
            .send(Event::ChangeWorkspace(id, true));

        Ok(())
    })
    .to_owned()
}
