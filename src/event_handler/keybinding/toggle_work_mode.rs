use std::sync::Arc;

use parking_lot::Mutex;

use crate::workspace::change_workspace;
use crate::{bar, keybindings::KbManager, popup};
use crate::{info, AppState};

pub fn initialize(
    state: &AppState,
    kb_manager: Arc<Mutex<KbManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if state.work_mode {
        turn_work_mode_on(state, kb_manager)?;
    }

    Ok(())
}

pub fn turn_work_mode_off(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    state.window_event_listener.stop();

    popup::cleanup();

    if state.config.display_app_bar {
        bar::close::close();
    }

    if state.config.remove_task_bar {
        state.show_taskbars();
    }

    state.cleanup()?;

    Ok(())
}

pub fn turn_work_mode_on(
    state: &AppState,
    kb_manager: Arc<Mutex<KbManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if state.config.remove_task_bar {
        info!("Hiding taskbar");
        state.hide_taskbars();
    }
    if state.config.display_app_bar {
        bar::create::create(state, kb_manager);
    }

    info!("Registering windows event handler");
    state.window_event_listener.start();

    change_workspace(1, false);

    Ok(())
}

pub fn handle(
    state: &mut AppState,
    kb_manager: Arc<Mutex<KbManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if state.work_mode {
        turn_work_mode_off(state)?;
    } else {
        turn_work_mode_on(state, kb_manager)?;
    }

    state.work_mode = !state.work_mode;

    Ok(())
}
