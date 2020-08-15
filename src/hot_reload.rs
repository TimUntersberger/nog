use crate::{
    bar, config::Config, display::get_display_by_hmonitor, keybindings, startup, task_bar, CONFIG,
    DISPLAYS, GRIDS, WORKSPACE_ID, WORK_MODE,
};

pub fn update_config(new_config: Config) -> Result<(), Box<dyn std::error::Error>> {
    keybindings::unregister();

    let config = CONFIG.lock().unwrap().clone();
    let work_mode = *WORK_MODE.lock().unwrap();
    let mut draw_app_bar = false;

    if work_mode {
        if config.display_app_bar && new_config.display_app_bar {
            if config.bar != new_config.bar || config.light_theme != new_config.light_theme {
                bar::close::close();
                draw_app_bar = true;
            }
        } else if config.display_app_bar && !new_config.display_app_bar {
            bar::close::close();

            for d in DISPLAYS.lock().unwrap().iter_mut() {
                d.bottom += config.bar.height;
            }

            for grid in GRIDS.lock().unwrap().iter_mut() {
                grid.display = get_display_by_hmonitor(grid.display.hmonitor);
            }
        } else if !config.display_app_bar && new_config.display_app_bar {
            draw_app_bar = true;

            for d in DISPLAYS.lock().unwrap().iter_mut() {
                d.bottom -= config.bar.height;
            }

            for grid in GRIDS.lock().unwrap().iter_mut() {
                grid.display = get_display_by_hmonitor(grid.display.hmonitor);
            }
        }
        if config.remove_task_bar && !new_config.remove_task_bar {
            task_bar::show();
        } else if !config.remove_task_bar && new_config.remove_task_bar {
            task_bar::hide();
        }
    }

    if config.remove_title_bar && !new_config.remove_title_bar {
        let mut grids = GRIDS.lock().unwrap();

        for grid in grids.iter_mut() {
            for tile in &mut grid.tiles {
                tile.window.reset_style()?;
                tile.window.update_style();
            }
        }
    } else if !config.remove_title_bar && new_config.remove_title_bar {
        let mut grids = GRIDS.lock().unwrap();

        for grid in grids.iter_mut() {
            for tile in &mut grid.tiles {
                tile.window.remove_title_bar();
                tile.window.update_style();
            }
        }
    }

    if config.launch_on_startup != new_config.launch_on_startup {
        startup::set_launch_on_startup(new_config.launch_on_startup)?;
    }

    *CONFIG.lock().unwrap() = new_config;

    if draw_app_bar {
        bar::create::create()?;
        bar::visibility::show();
    }

    keybindings::register()?;

    let mut grids = GRIDS.lock().unwrap();
    let grid = grids
        .iter_mut()
        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
        .unwrap();

    grid.draw_grid();

    Ok(())
}
