use std::sync::Arc;

use crate::{
    config::{rhai::engine, rule::Rule},
    event::Event,
    hot_reload::update_config,
    keybindings::{keybinding::Keybinding, keybinding_type::KeybindingType},
    tile_grid::store::Store,
    system::api,
    system::SystemResult,
    AppState,
};
use log::{debug, info};
use parking_lot::Mutex;

mod close_tile;
mod focus;
mod resize;
mod split;
mod swap;
mod swap_columns_rows;
mod move_in;
mod move_out;
mod toggle_floating_mode;
pub mod toggle_work_mode;

pub fn handle(state_arc: Arc<Mutex<AppState>>, kb: Keybinding) -> SystemResult {
    let mut state = state_arc.lock();
    let config = state.config.clone();
    if let KeybindingType::MoveWorkspaceToMonitor(_) = kb.typ {
        if !state.config.multi_monitor {
            return Ok(());
        }
    }

    info!("Received keybinding of type {:?}", kb.typ);
    let sender = state.event_channel.sender.clone();

    match kb.typ {
        KeybindingType::Launch(cmd) => {
            api::launch_program(cmd)?;
        }
        KeybindingType::MoveWorkspaceToMonitor(monitor) => {
            if let Some(_) = state.get_display_by_idx(monitor) {
                let display = state.get_current_display_mut();
                if let Some(grid) = display
                    .focused_grid_id
                    .and_then(|id| display.remove_grid_by_id(id))
                {
                    let new_display = state
                        .get_display_by_idx_mut(monitor)
                        .expect("Monitor with specified idx doesn't exist");

                    let id = grid.id;

                    new_display.grids.push(grid);
                    new_display.focus_workspace(&config, id)?;
                    state.workspace_id = id;
                }
            }
        }
        KeybindingType::CloseTile => close_tile::handle(&mut state)?,
        KeybindingType::MinimizeTile => {
            let grid = state.get_current_grid_mut().unwrap();
            grid.modify_focused_window(|window| {
                window.minimize()?;
                window.cleanup()
            })?;

            grid.close_focused();
            let display = state.get_current_display_mut();
            display.refresh_grid(&config)?;
        }
        KeybindingType::MoveToWorkspace(id) => {
            let grid = state.get_current_grid_mut().unwrap();
            let popped_window = grid.pop();
            Store::save(grid.id, grid.to_string()); // save modification

            popped_window.map(|window| {
                state.get_grid_by_id_mut(id).unwrap().push(window);
                state.change_workspace(id, false);
            });
        }
        KeybindingType::ChangeWorkspace(id) => state.change_workspace(id, false),
        KeybindingType::ToggleFloatingMode => toggle_floating_mode::handle(&mut state)?,
        KeybindingType::ToggleFullscreen => {
            let display = state.get_current_display_mut();
            display.get_focused_grid_mut().unwrap().toggle_fullscreen();
            display.refresh_grid(&config)?;
        }
        KeybindingType::ToggleMode(mode) => {
            if state.keybindings_manager.get_mode() == Some(mode.clone()) {
                info!("Disabling {} mode", mode);
                state.keybindings_manager.leave_mode();
            } else {
                info!("Enabling {} mode", mode);
                state.keybindings_manager.enter_mode(&mode);
            }
        }
        KeybindingType::ToggleWorkMode => {
            drop(state);
            toggle_work_mode::handle(state_arc.clone())?
        }
        KeybindingType::IncrementConfig(field, value) => {
            let new_config = state.config.increment_field(&field, value);
            drop(state);
            update_config(state_arc.clone(), new_config)?;
        }
        KeybindingType::DecrementConfig(field, value) => {
            let new_config = state.config.decrement_field(&field, value);
            drop(state);
            update_config(state_arc.clone(), new_config)?;
        }
        KeybindingType::ToggleConfig(field) => {
            let new_config = state.config.toggle_field(&field);
            drop(state);
            update_config(state_arc.clone(), new_config)?;
        }
        KeybindingType::Resize(direction, amount) => resize::handle(&mut state, direction, amount)?,
        KeybindingType::Focus(direction) => focus::handle(&mut state, direction)?,
        KeybindingType::Swap(direction) => swap::handle(&mut state, direction)?,
        KeybindingType::SwapColumnsAndRows => swap_columns_rows::handle(&mut state)?,
        KeybindingType::MoveIn(direction) => move_in::handle(&mut state, direction)?,
        KeybindingType::MoveOut(direction) => move_out::handle(&mut state, direction)?,
        KeybindingType::Quit => sender.send(Event::Exit).expect("Failed to send exit event"),
        KeybindingType::Split(direction) => split::handle(&mut state, direction)?,
        KeybindingType::ResetColumn => {
            let display = state.get_current_display_mut();
            if let Some(g) = display.get_focused_grid_mut() {
                if !config.ignore_fullscreen_actions || !g.is_fullscreened() {
                    g.reset_column();
                }
            }
            display.refresh_grid(&config)?;
        }
        KeybindingType::ResetRow => {
            let display = state.get_current_display_mut();
            if let Some(g) = display.get_focused_grid_mut() {
                if !config.ignore_fullscreen_actions || !g.is_fullscreened() {
                    g.reset_row();
                }
            }
            display.refresh_grid(&config)?;
        }
        KeybindingType::Callback(idx) => {
            drop(state);
            engine::call(idx)
        }
        KeybindingType::IgnoreTile => {
            if let Some(window) = state.get_current_grid().unwrap().get_focused_window() {
                let mut rule = Rule::default();

                let process_name = window.get_process_name();
                let pattern = format!("^{}$", process_name);

                debug!("Adding rule with pattern {}", pattern);

                rule.pattern = regex::Regex::new(&pattern).expect("Failed to build regex");
                rule.manage = false;

                state.additonal_rules.push(rule);

                toggle_floating_mode::handle(&mut state)?;
            }
        }
    };

    Ok(())
}
