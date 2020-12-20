use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    bar, config::Config, keybindings::KbManager, startup, system::SystemResult, task_bar, AppState,
};

pub fn update_config(state_arc: Arc<Mutex<AppState>>, new_config: Config) -> SystemResult {
    state_arc.lock().keybindings_manager.stop();

    let mut state = state_arc.lock();
    let work_mode = state.work_mode;
    let old_config = state.config.clone();
    let mut draw_app_bar = false;
    let mut close_app_bars = false;

    state.config = new_config;
    state.keybindings_manager = KbManager::new(
        state.config.keybindings.clone(),
        state.config.mode_handlers.clone(),
    );

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
            grid.modify_windows(|window| {
                window.reset_style();
                window
                    .update_style()
                    .expect("Failed to update style of window");
                Ok(())
            })?;
        }
    } else if !old_config.remove_title_bar && state.config.remove_title_bar {
        let use_border = old_config.use_border;
        for grid in state.get_grids_mut() {
            grid.modify_windows(|window| {
                window.remove_title_bar(use_border)?;
                window
                    .update_style()
                    .expect("Failed to update style of window");
                Ok(())
            })?;
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
            grid.draw_grid(d, &state.config)?;
        }
    }

    Ok(())
}
