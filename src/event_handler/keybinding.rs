use std::sync::Arc;

use crate::{
    config::{rhai::engine, rule::Rule},
    event::Event,
    hot_reload::update_config,
    keybindings::{keybinding::Keybinding, keybinding_type::KeybindingType},
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
    let display = state.get_current_display_mut();

    match kb.typ {
        KeybindingType::Launch(cmd) => {
            api::launch_program(cmd)?;
        }
        KeybindingType::MoveWorkspaceToMonitor(monitor) => {
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
        KeybindingType::CloseTile => close_tile::handle(&mut state)?,
        KeybindingType::MinimizeTile => {
            let grid = state.get_current_grid_mut().unwrap();
            if let Some(tile) = grid.get_focused_tile_mut() {
                let id = tile.window.id;

                tile.window.minimize()?;
                tile.window.cleanup()?;

                grid.close_tile_by_window_id(id);
            }
        }
        KeybindingType::MoveToWorkspace(id) => {
            let grid = state.get_current_grid_mut().unwrap();
            grid.focused_window_id
                .and_then(|id| grid.close_tile_by_window_id(id))
                .map(|tile| {
                    state.get_grid_by_id_mut(id).unwrap().split(tile.window);
                    state.change_workspace(id, false);
                });
        }
        KeybindingType::ChangeWorkspace(id) => state.change_workspace(id, false),
        KeybindingType::ToggleFloatingMode => toggle_floating_mode::handle(&mut state)?,
        KeybindingType::ToggleFullscreen => {
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
        KeybindingType::Quit => sender.send(Event::Exit).expect("Failed to send exit event"),
        KeybindingType::Split(direction) => split::handle(&mut state, direction)?,
        KeybindingType::ResetColumn => {
            if let Some(g) = display.get_focused_grid_mut() {
                g.reset_column();
            }
            display.refresh_grid(&config)?;
        }
        KeybindingType::ResetRow => {
            if let Some(g) = display.get_focused_grid_mut() {
                g.reset_row();
            }
            display.refresh_grid(&config)?;
        }
        KeybindingType::Callback(idx) => engine::call(idx),
        KeybindingType::IgnoreTile => {
            if let Some(tile) = state.get_current_grid().unwrap().get_focused_tile() {
                let mut rule = Rule::default();

                let process_name = tile.window.get_process_name();
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
