use std::{sync::Arc, thread, time::Duration};

use parking_lot::Mutex;

use crate::{bar, config::Config, startup, system::SystemResult, task_bar, AppState};

pub fn update_config(state_arc: Arc<Mutex<AppState>>, new_config: Config) -> SystemResult {
    let mut state = state_arc.lock();
    let work_mode = state.work_mode;
    let mut draw_app_bar = false;
    let mut close_app_bars = false;
    let old_config = state.config.clone();

    state.config = new_config;

    state.keybindings_manager.stop();

    if work_mode {
        if old_config.remove_task_bar && !state.config.remove_task_bar {
            state.show_taskbars();
            close_app_bars = true;
            draw_app_bar = state.config.display_app_bar;
        } else if !old_config.remove_task_bar && state.config.remove_task_bar {
            state.hide_taskbars();
            close_app_bars = true;
            draw_app_bar = state.config.display_app_bar;
        }

        if old_config.display_app_bar && state.config.display_app_bar {
            if old_config.bar != state.config.bar
                || old_config.light_theme != state.config.light_theme
            {
                close_app_bars = true;
                draw_app_bar = true;
            }
        } else if old_config.display_app_bar && !state.config.display_app_bar {
            close_app_bars = true;
        } else if !old_config.display_app_bar && state.config.display_app_bar {
            draw_app_bar = true;
        }
    }

    //TODO: handle multi monitor change

    if old_config.remove_title_bar && !state.config.remove_title_bar {
        for grid in state.get_grids_mut().iter_mut() {
            for tile in &mut grid.tiles {
                tile.window.reset_style();
                tile.window
                    .update_style()
                    .expect("Failed to update style of window");
            }
        }
    } else if !old_config.remove_title_bar && state.config.remove_title_bar {
        let use_border = old_config.use_border;
        for grid in state.get_grids_mut() {
            for tile in &mut grid.tiles {
                tile.window.remove_title_bar(use_border)?;
                tile.window
                    .update_style()
                    .expect("Failed to update style of window");
            }
        }
    }

    if old_config.launch_on_startup != state.config.launch_on_startup {
        startup::set_launch_on_startup(state.config.launch_on_startup);
    }

    if close_app_bars {
        drop(state);
        bar::close_all(state_arc.clone());
        state = state_arc.lock();
    }

    if draw_app_bar {
        drop(state);
        bar::create::create(state_arc.clone());
        state = state_arc.lock();
    }

    state.keybindings_manager.start(state_arc.clone());

    for d in state.displays.iter() {
        if let Some(grid) = d.get_focused_grid() {
            grid.draw_grid(d, &old_config)?;
        }
    }

    Ok(())
}
