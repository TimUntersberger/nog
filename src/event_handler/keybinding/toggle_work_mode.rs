use crate::app_bar;
use crate::hot_key_manager;
use crate::task_bar;
use crate::unmanage_everything;
use crate::win_event_handler;
use crate::CONFIG;
use crate::DISPLAY;
use crate::WORK_MODE;

pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let work_mode = *WORK_MODE.lock().unwrap();

    if work_mode {
        win_event_handler::unregister()?;
        hot_key_manager::disable();

        if CONFIG.display_app_bar {
            app_bar::close();
        }
        if CONFIG.remove_task_bar {
            task_bar::show();
        }

        unmanage_everything()?;
    } else {
        win_event_handler::register()?;
        hot_key_manager::enable();

        if CONFIG.display_app_bar {
            app_bar::create(&*DISPLAY.lock().unwrap()).expect("Failed to create app bar");
        }
        if CONFIG.remove_task_bar {
            task_bar::hide();
        }
    }

    *WORK_MODE.lock().unwrap() = !work_mode;
    Ok(())
}
