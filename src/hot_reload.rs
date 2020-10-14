use std::sync::Arc;

use parking_lot::Mutex;

use crate::{bar, config::Config, startup, task_bar, AppState, system::SystemResult};

pub fn update_config(
    state_arc: Arc<Mutex<AppState>>,
    new_config: Config,
) -> SystemResult {
    let mut state = state_arc.lock();
    let work_mode = state.work_mode;
    let mut draw_app_bar = false;
    let mut close_app_bars = false;

    state.keybindings_manager.stop();

    if work_mode {
        if state.config.remove_task_bar && !new_config.remove_task_bar {
            state.show_taskbars();
            close_app_bars = true;
            draw_app_bar = new_config.display_app_bar;
        } else if !state.config.remove_task_bar && new_config.remove_task_bar {
            task_bar::hide_taskbars(&mut state);
            close_app_bars = true;
            draw_app_bar = new_config.display_app_bar;
        }

        if state.config.display_app_bar && new_config.display_app_bar {
            if state.config.bar != new_config.bar
                || state.config.light_theme != new_config.light_theme
            {
                close_app_bars = true;
                draw_app_bar = true;
            }
        } else if state.config.display_app_bar && !new_config.display_app_bar {
            close_app_bars = true;
        } else if !state.config.display_app_bar && new_config.display_app_bar {
            draw_app_bar = true;
        }
    }

    if state.config.remove_title_bar && !new_config.remove_title_bar {
        for grid in state.get_grids_mut().iter_mut() {
            for tile in &mut grid.tiles {
                tile.window.reset_style();
                tile.window.update_style().expect("Failed to update style of window");
            }
        }
    } else if !state.config.remove_title_bar && new_config.remove_title_bar {
        let use_border = state.config.use_border;
        for grid in state.get_grids_mut() {
            for tile in &mut grid.tiles {
                tile.window.remove_title_bar(use_border)?;
                tile.window.update_style().expect("Failed to update style of window");
            }
        }
    }

    if state.config.launch_on_startup != new_config.launch_on_startup {
        startup::set_launch_on_startup(new_config.launch_on_startup);
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
            grid.draw_grid(d, &state.config)?;
        }
    }

    state.config = new_config;

    Ok(())
}
