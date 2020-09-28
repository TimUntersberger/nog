use crate::{
    display::with_display_by_idx, event::Event, CHANNEL, CONFIG, GRIDS, VISIBLE_WORKSPACES,
    WORKSPACE_ID,
};
use log::debug;

pub fn is_visible_workspace(id: i32) -> bool {
    VISIBLE_WORKSPACES.lock().values().any(|v| *v == id)
}

pub fn change_workspace(id: i32, ignore_monitor_setting: bool) {
    let mut grids = GRIDS.lock();

    let workspace_settings = CONFIG.lock().workspace_settings.clone();

    let (new_grid_idx, mut new_grid) = grids
        .iter_mut()
        .enumerate()
        .find(|(_, g)| g.id == id)
        .map(|(i, g)| (i, g.clone()))
        .unwrap();

    if !ignore_monitor_setting {
        if new_grid.tiles.is_empty() {
            if let Some(setting) = workspace_settings.iter().find(|s| s.id == id) {
                if setting.monitor != -1 {
                    new_grid.display = with_display_by_idx(setting.monitor, |d| d.unwrap().clone());
                }
            }
        }
    }

    let mut visible_workspaces = VISIBLE_WORKSPACES.lock();

    debug!("Drawing the workspace");
    new_grid.draw_grid();
    debug!("Showing the workspace");
    new_grid.show();

    if let Some(id) = visible_workspaces.insert(new_grid.display.id, new_grid.id) {
        if new_grid.id != id {
            if let Some(grid) = grids.iter().find(|g| g.id == id) {
                debug!("Hiding the current workspace");
                grid.hide();
            } else {
                debug!("Workspace is already visible");
            }
        }
    }

    debug!("Updating workspace id of monitor");
    grids.remove(new_grid_idx);
    grids.insert(new_grid_idx, new_grid);

    *WORKSPACE_ID.lock() = id;

    debug!("Sending redraw-app-bar event");
    CHANNEL
        .sender
        .clone()
        .send(Event::RedrawAppBar)
        .expect("Failed to send redraw-app-bar event");
}
