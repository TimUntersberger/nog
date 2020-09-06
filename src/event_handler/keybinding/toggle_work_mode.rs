use crate::bar;
use crate::task_bar;
use crate::unmanage_everything;
use crate::win_event_handler;
use crate::CONFIG;
use crate::{popup, WORK_MODE};
use crate::info;
use crate::workspace::{change_workspace};

pub fn initialize() -> Result<(), Box<dyn std::error::Error>> {
    if *WORK_MODE.lock().unwrap() {
        let display_app_bar = CONFIG.lock().unwrap().display_app_bar;
        let remove_task_bar = CONFIG.lock().unwrap().remove_task_bar;
        turn_work_mode_on(display_app_bar, remove_task_bar)?;
    }

    Ok(())
}

pub fn turn_work_mode_off(
    display_app_bar: bool,
    remove_task_bar: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    win_event_handler::unregister()?;

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
) -> Result<(), Box<dyn std::error::Error>> {
    if remove_task_bar {
        info!("Hiding taskbar");
        task_bar::hide_taskbars();
    }
    if display_app_bar {
        bar::create::create().expect("Failed to create app bar");
    }

    info!("Registering windows event handler");
    win_event_handler::register()?;

    info!("Initializing bars");

    change_workspace(1, false).expect("Failed to change workspace to ID@1");

    Ok(())
}

pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let work_mode = *WORK_MODE.lock().unwrap();
    let display_app_bar = CONFIG.lock().unwrap().display_app_bar;
    let remove_task_bar = CONFIG.lock().unwrap().remove_task_bar;

    if work_mode {
        turn_work_mode_off(display_app_bar, remove_task_bar)?;
    } else {
        turn_work_mode_on(display_app_bar, remove_task_bar)?;
    }

    *WORK_MODE.lock().unwrap() = !work_mode;

    Ok(())
}
