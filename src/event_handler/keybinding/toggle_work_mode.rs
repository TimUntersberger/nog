use crate::{bar, keybindings::KbManager, popup};
use crate::{info, AppState};
use parking_lot::Mutex;
use std::sync::Arc;

pub fn initialize(
    state_arc: Arc<Mutex<AppState>>,
    kb_manager: Arc<Mutex<KbManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if state_arc.lock().work_mode {
        turn_work_mode_on(state_arc, kb_manager)?;
    }

    Ok(())
}

pub fn turn_work_mode_off(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    state.window_event_listener.stop();

    popup::cleanup();

    if state.config.display_app_bar {
        // TODO: close bar
    }

    if state.config.remove_task_bar {
        state.show_taskbars();
    }

    state.cleanup()?;

    Ok(())
}

pub fn turn_work_mode_on(
    state_arc: Arc<Mutex<AppState>>,
    kb_manager: Arc<Mutex<KbManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = state_arc.lock();

    if state.config.remove_task_bar {
        info!("Hiding taskbar");
        state.hide_taskbars();
    }

    if state.config.display_app_bar {
        drop(state);
        bar::create::create(state_arc, kb_manager);
    }

    let state = state_arc.lock();

    info!("Registering windows event handler");
    state.window_event_listener.start();

    state.change_workspace(1, false);

    Ok(())
}

pub fn handle(
    state_arc: Arc<Mutex<AppState>>,
    kb_manager: Arc<Mutex<KbManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = state_arc.lock();

    state.work_mode = !state.work_mode;

    if !state.work_mode {
        turn_work_mode_off(&state)?;
    } else {
        drop(state);
        turn_work_mode_on(state_arc, kb_manager)?;
    }

    Ok(())
}
