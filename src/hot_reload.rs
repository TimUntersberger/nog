use std::sync::Arc;

use keybindings::KbManager;
use parking_lot::Mutex;

use crate::{bar, config::Config, keybindings, startup, task_bar, AppState};

pub fn update_config(
    state: &AppState,
    kb_manager: Arc<Mutex<KbManager>>,
    new_config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    //TODO: unregister keybindings
    let work_mode = state.work_mode;
    let config = state.config;
    let mut draw_app_bar = false;

    if work_mode {
        if config.remove_task_bar && !new_config.remove_task_bar {
            state.show_taskbars();
            bar::close::close();
            draw_app_bar = new_config.display_app_bar;
        } else if !config.remove_task_bar && new_config.remove_task_bar {
            task_bar::hide_taskbars();
            bar::close::close();
            draw_app_bar = new_config.display_app_bar;
        }

        if config.display_app_bar && new_config.display_app_bar {
            if config.bar != new_config.bar || config.light_theme != new_config.light_theme {
                bar::close::close();
                draw_app_bar = true;
            }
        } else if config.display_app_bar && !new_config.display_app_bar {
            bar::close::close();
        } else if !config.display_app_bar && new_config.display_app_bar {
            draw_app_bar = true;
        }
    }

    if config.remove_title_bar && !new_config.remove_title_bar {
        for grid in state.get_grids().iter_mut() {
            for tile in &mut grid.tiles {
                tile.window.reset_style();
                tile.window.update_style();
            }
        }
    } else if !config.remove_title_bar && new_config.remove_title_bar {
        for grid in state.get_grids().iter_mut() {
            for tile in &mut grid.tiles {
                tile.window.remove_title_bar();
                tile.window.update_style();
            }
        }
    }

    if config.launch_on_startup != new_config.launch_on_startup {
        startup::set_launch_on_startup(new_config.launch_on_startup);
    }

    state.config = new_config;

    if draw_app_bar {
        bar::create::create(state, kb_manager);
        bar::visibility::show();
    }

    //TODO: register keybindings

    for d in state.displays {
        // TODO: redraw visible grids
    }

    Ok(())
}
