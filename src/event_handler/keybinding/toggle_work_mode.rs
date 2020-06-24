use crate::CONFIG;
use crate::WORK_MODE;
use crate::app_bar;
use crate::task_bar;
use crate::unmanage_everything;

use log::debug;

pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let work_mode = *WORK_MODE.lock().unwrap();

    if work_mode {
        if CONFIG.display_app_bar {
            app_bar::hide();
        }
        if CONFIG.remove_task_bar {
            task_bar::show();
        }

        unmanage_everything()?;
    } else {
        if CONFIG.display_app_bar {
            app_bar::show();
        }
        if CONFIG.remove_task_bar {
            task_bar::hide();
        }
    }

    *WORK_MODE.lock().unwrap() = !work_mode;
    Ok(())
}
