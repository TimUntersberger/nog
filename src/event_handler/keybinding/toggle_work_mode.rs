use crate::DISPLAY;
use crate::CONFIG;
use crate::WORK_MODE;
use crate::app_bar;
use crate::task_bar;
use crate::win_event_handler;
use crate::hot_key_manager;
use crate::unmanage_everything;

use log::debug;

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
            app_bar::create(&*DISPLAY.lock().unwrap());
        }
        if CONFIG.remove_task_bar {
            task_bar::hide();
        }
    }

    *WORK_MODE.lock().unwrap() = !work_mode;
    Ok(())
}
