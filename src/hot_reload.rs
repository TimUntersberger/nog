use std::sync::Arc;

use keybindings::KbManager;
use parking_lot::Mutex;

use crate::{bar, config::Config, keybindings, startup, task_bar, AppState};

pub fn update_config(
    state_arc: Arc<Mutex<AppState>>,
    kb_manager: Arc<Mutex<KbManager>>,
    new_config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    //TODO: unregister keybindings
    let mut state = state_arc.lock();
    let work_mode = state.work_mode;
    let mut draw_app_bar = false;

    if work_mode {
        if state.config.remove_task_bar && !new_config.remove_task_bar {
            state.show_taskbars();
            for d in state.displays.iter() {
                if let Some(b) = d.appbar.as_ref() {
                    b.window.close();
                }
            }
            draw_app_bar = new_config.display_app_bar;
        } else if !state.config.remove_task_bar && new_config.remove_task_bar {
            task_bar::hide_taskbars(&mut state);
            for d in state.displays.iter() {
                if let Some(b) = d.appbar.as_ref() {
                    b.window.close();
                }
            }
            draw_app_bar = new_config.display_app_bar;
        }

        if state.config.display_app_bar && new_config.display_app_bar {
            if state.config.bar != new_config.bar
                || state.config.light_theme != new_config.light_theme
            {
                for d in state.displays.iter() {
                    if let Some(b) = d.appbar.as_ref() {
                        b.window.close();
                    }
                }
                draw_app_bar = true;
            }
        } else if state.config.display_app_bar && !new_config.display_app_bar {
            for d in state.displays.iter() {
                if let Some(b) = d.appbar.as_ref() {
                    b.window.close();
                }
            }
        } else if !state.config.display_app_bar && new_config.display_app_bar {
            draw_app_bar = true;
        }
    }

    if state.config.remove_title_bar && !new_config.remove_title_bar {
        for grid in state.get_grids_mut().iter_mut() {
            for tile in &mut grid.tiles {
                tile.window.reset_style();
                tile.window.update_style();
            }
        }
    } else if !state.config.remove_title_bar && new_config.remove_title_bar {
        let use_border = state.config.use_border;
        for grid in state.get_grids_mut() {
            for tile in &mut grid.tiles {
                tile.window.remove_title_bar(use_border);
                tile.window.update_style();
            }
        }
    }

    if state.config.launch_on_startup != new_config.launch_on_startup {
        startup::set_launch_on_startup(new_config.launch_on_startup);
    }

    //TODO: register keybindings

    for d in state.displays.iter() {
        if let Some(grid) = d.get_focused_grid() {
            grid.draw_grid(d, &state.config);
        }
    }

    state.config = new_config;

    if draw_app_bar {
        drop(state);
        bar::create::create(state_arc.clone(), kb_manager);
    }

    Ok(())
}
