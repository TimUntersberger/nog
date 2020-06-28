use crate::app_bar;
use crate::task_bar;
use crate::unmanage_everything;
use crate::win_event_handler;
use crate::CONFIG;
use crate::DISPLAY;
use crate::WORK_MODE;

pub fn turn_work_mode_off(display_app_bar: bool, remove_task_bar: bool) -> Result<(), Box<dyn std::error::Error>> {
    win_event_handler::unregister()?;

    if display_app_bar {
        app_bar::close();
    }
    
    if remove_task_bar {
        task_bar::show();
    }

    unmanage_everything()?; 
    Ok(())
}

pub fn turn_work_mode_on(display_app_bar: bool, remove_task_bar: bool) -> Result<(), Box<dyn std::error::Error>> {
    win_event_handler::register()?;
    if display_app_bar {
        app_bar::create(&*DISPLAY.lock().unwrap()).expect("Failed to create app bar");
    }
    if remove_task_bar {
        task_bar::hide();
    }
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
