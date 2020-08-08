use crate::{
    display::get_display_by_idx, event::Event, util, CHANNEL, CONFIG,
    GRIDS, VISIBLE_WORKSPACES, WORKSPACE_ID,
};
use log::debug;

pub struct Workspace {
    pub id: i32,
    pub visible: bool,
}

impl Workspace {
    pub fn new(id: i32) -> Self {
        Self { id, visible: false }
    }
}

pub fn is_visible_workspace(id: i32) -> bool {
    VISIBLE_WORKSPACES
        .lock()
        .unwrap()
        .values()
        .any(|v| *v == id)
}

pub fn change_workspace(id: i32) -> Result<(), util::WinApiResultError> {
    let mut grids = GRIDS.lock().unwrap();

    let workspace_settings = CONFIG.lock().unwrap().workspace_settings.clone();

    let (new_grid_idx, mut new_grid) = grids
        .iter_mut()
        .enumerate()
        .find(|(_, g)| g.id == id)
        .map(|(i, g)| (i, g.clone()))
        .unwrap();

    if let Some(setting) = workspace_settings.iter().find(|s| s.id == id) {
        new_grid.display = get_display_by_idx(setting.monitor);
    }

    let mut visible_workspaces = VISIBLE_WORKSPACES.lock().unwrap();

    debug!("Drawing the workspace");
    new_grid.draw_grid();
    debug!("Showing the workspace");
    new_grid.show();

    if let Some(id) = visible_workspaces.insert(new_grid.display.hmonitor, new_grid.id) {
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

    *WORKSPACE_ID.lock().unwrap() = id;

    debug!("Sending redraw-app-bar event");
    CHANNEL
        .sender
        .clone()
        .send(Event::RedrawAppBar)
        .expect("Failed to send redraw-app-bar event");

    Ok(())
}
