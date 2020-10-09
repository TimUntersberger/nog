use crate::{bar, keybindings::KbManager, popup};
use crate::{info, AppState};
use parking_lot::Mutex;
use std::sync::Arc;

pub fn initialize(state_arc: Arc<Mutex<AppState>>) -> Result<(), Box<dyn std::error::Error>> {
    if state_arc.lock().work_mode {
        turn_work_mode_on(state_arc)?;
    }

    Ok(())
}

pub fn turn_work_mode_off(state_arc: Arc<Mutex<AppState>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = state_arc.lock();
    state.window_event_listener.stop();

    popup::cleanup();

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

pub fn turn_work_mode_on(
    state_arc: Arc<Mutex<AppState>>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    state.change_workspace(1, false);

    info!("Registering windows event handler");
    state.window_event_listener.start(&state.event_channel);

    Ok(())
}

pub fn handle(state_arc: Arc<Mutex<AppState>>) -> Result<(), Box<dyn std::error::Error>> {
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
