use std::sync::Arc;

use parking_lot::Mutex;

use crate::info;
use crate::task_bar;
use crate::unmanage_everything;
use crate::workspace::change_workspace;
use crate::CONFIG;
use crate::WIN_EVENT_LISTENER;
use crate::{bar, keybindings::KbManager};
use crate::{popup, WORK_MODE};

pub fn initialize(kb_manager: Arc<Mutex<KbManager>>) -> Result<(), Box<dyn std::error::Error>> {
    if *WORK_MODE.lock() {
        let display_app_bar = CONFIG.lock().display_app_bar;
        let remove_task_bar = CONFIG.lock().remove_task_bar;
        turn_work_mode_on(display_app_bar, remove_task_bar, kb_manager)?;
    }

    Ok(())
}

pub fn turn_work_mode_off(
    display_app_bar: bool,
    remove_task_bar: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    WIN_EVENT_LISTENER.stop();

    popup::close();

    if display_app_bar {
        bar::close::close();
    }

    if remove_task_bar {
        task_bar::show_taskbars();
    }

    unmanage_everything()?;
    Ok(())
}

pub fn turn_work_mode_on(
    display_app_bar: bool,
    remove_task_bar: bool,
    kb_manager: Arc<Mutex<KbManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if remove_task_bar {
        info!("Hiding taskbar");
        task_bar::hide_taskbars();
    }
    if display_app_bar {
        bar::create::create(kb_manager);
    }

    info!("Registering windows event handler");
    WIN_EVENT_LISTENER.start();

    info!("Initializing bars");

    change_workspace(1, false);

    Ok(())
}

pub fn handle(kb_manager: Arc<Mutex<KbManager>>) -> Result<(), Box<dyn std::error::Error>> {
    let work_mode = *WORK_MODE.lock();
    let display_app_bar = CONFIG.lock().display_app_bar;
    let remove_task_bar = CONFIG.lock().remove_task_bar;

    if work_mode {
        turn_work_mode_off(display_app_bar, remove_task_bar)?;
    } else {
        turn_work_mode_on(display_app_bar, remove_task_bar, kb_manager)?;
    }

    *WORK_MODE.lock() = !work_mode;

    Ok(())
}
