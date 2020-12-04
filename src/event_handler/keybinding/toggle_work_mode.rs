use crate::{bar, popup, system::SystemResult, tile_grid::store::Store};
use crate::{info, AppState};
use log::error;
use parking_lot::Mutex;
use std::sync::Arc;

pub fn initialize(state_arc: Arc<Mutex<AppState>>) -> Result<(), Box<dyn std::error::Error>> {
    if state_arc.lock().work_mode {
        turn_work_mode_on(state_arc)?;
    }

    Ok(())
}

pub fn turn_work_mode_off(state_arc: Arc<Mutex<AppState>>) -> SystemResult {
    let mut state = state_arc.lock();
    state.window_event_listener.stop();

    popup::cleanup()?;

    for display in state.displays.iter() {
        for grid in display.grids.iter() {
            Store::save(grid.id, grid.to_string());
        }
    }

    if state.config.display_app_bar {
        drop(state);
        bar::close_all(state_arc.clone());
        state = state_arc.lock();
    }

    if state.config.remove_task_bar {
        state.show_taskbars();
    }

    state.cleanup()?;

    Ok(())
}

pub fn turn_work_mode_on(state_arc: Arc<Mutex<AppState>>) -> SystemResult {
    let mut state = state_arc.lock();

    if state.config.remove_task_bar {
        info!("Hiding taskbar");
        state.hide_taskbars();
    }

    if state.config.display_app_bar {
        drop(state);
        bar::create::create(state_arc.clone());
        state = state_arc.lock();
    }

    let mut focused_workspaces = Vec::<i32>::new();
    let remove_title_bar = state.config.remove_title_bar;
    let use_border = state.config.use_border;
    let stored_grids: Vec<String> = Store::load();
    for display in state.displays.iter_mut() {
        for grid in display.grids.iter_mut() {
            if let Some(stored_grid) = stored_grids.get((grid.id - 1) as usize) {
                grid.from_string(stored_grid);
                Store::save(grid.id, grid.to_string());

                if remove_title_bar {
                    if let Err(e) = grid.modify_windows(|window| {
                                        window.remove_title_bar(use_border)?;
                                        Ok(())
                                    }) {
                        error!("Error while removing title bar {:?}", e);
                    }
                }

                grid.hide(); // hides all the windows just loaded into the grid
            }
        }

        if let Some(id) = display.focused_grid_id {
            focused_workspaces.push(id); 
        }
    }

    if !focused_workspaces.is_empty() {  // re-focus to show each display's focused workspace
        for id in focused_workspaces.iter().rev() {
            state.change_workspace(*id, false);
        }
    } else { // otherwise just focus first workspace
        state.change_workspace(1, false);
    }

    info!("Registering windows event handler");
    state.window_event_listener.start(&state.event_channel);

    Ok(())
}

pub fn handle(state_arc: Arc<Mutex<AppState>>) -> SystemResult {
    let mut state = state_arc.lock();

    state.work_mode = !state.work_mode;

    let work_mode = state.work_mode;

    drop(state);

    if !work_mode {
        turn_work_mode_off(state_arc)?;
    } else {
        turn_work_mode_on(state_arc)?;
    }

    Ok(())
}
