use std::sync::Arc;

use crate::{
    config::{rhai::engine, rule::Rule},
    event::Event,
    hot_reload::update_config,
    keybindings::{self, keybinding::Keybinding, keybinding_type::KeybindingType},
    system::api,
    AppState,
};
use keybindings::KbManager;
use log::{debug, info};
use parking_lot::Mutex;

mod close_tile;
mod focus;
mod resize;
mod split;
mod swap;
mod toggle_floating_mode;
pub mod toggle_work_mode;

pub fn handle(
    state_arc: Arc<Mutex<AppState>>,
    kb_manager: Arc<Mutex<KbManager>>,
    kb: Keybinding,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = state_arc.lock();
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
            let mut display = state.get_current_display_mut();
            if let Some(id) = display.focused_grid_id {
                let grid = display.remove_grid_by_id(id).unwrap();
                let new_display = state
                    .get_display_by_idx(monitor)
                    .expect("Monitor with specified idx doesn't exist");

                new_display.grids.push(grid);

                new_display.focus_workspace(&state, id);
                state.workspace_id = id;
            }
        }
        KeybindingType::CloseTile => close_tile::handle(&state)?,
        KeybindingType::MinimizeTile => {
            let grid = state.get_current_grid_mut().unwrap();
            if let Some(tile) = grid.get_focused_tile_mut() {
                let id = tile.window.id;

                tile.window.minimize();
                tile.window.cleanup();

                grid.close_tile_by_window_id(id);
            }
        }
        KeybindingType::MoveToWorkspace(id) => {
            let grid = state.get_current_grid().unwrap();
            grid.focused_window_id
                .and_then(|id| grid.close_tile_by_window_id(id))
                .map(|tile| {
                    state.get_grid_by_id_mut(id).unwrap().split(tile.window);
                    state.change_workspace(id, false);
                });
        }
        KeybindingType::ChangeWorkspace(id) => state.change_workspace(id, false),
        KeybindingType::ToggleFloatingMode => toggle_floating_mode::handle(&state)?,
        KeybindingType::ToggleFullscreen => state
            .get_current_grid()
            .unwrap()
            .toggle_fullscreen(display, &state.config),
        KeybindingType::ToggleMode(mode) => {
            if kb_manager.lock().get_mode() == Some(mode.clone()) {
                info!("Disabling {} mode", mode);
                kb_manager.lock().leave_mode();
            } else {
                info!("Enabling {} mode", mode);
                kb_manager.lock().enter_mode(&mode);
            }
        }
        KeybindingType::ToggleWorkMode => toggle_work_mode::handle(state_arc, kb_manager)?,
        KeybindingType::IncrementConfig(field, value) => {
            let new_config = state.config.increment_field(&field, value);
            drop(state);
            update_config(state_arc.clone(), kb_manager, new_config)?;
        }
        KeybindingType::DecrementConfig(field, value) => {
            let new_config = state.config.decrement_field(&field, value);
            drop(state);
            update_config(state_arc.clone(), kb_manager, new_config)?;
        }
        KeybindingType::ToggleConfig(field) => {
            let new_config = state.config.toggle_field(&field);
            drop(state);
            update_config(state_arc.clone(), kb_manager, new_config)?;
        }
        KeybindingType::Resize(direction, amount) => resize::handle(&mut state, direction, amount)?,
        KeybindingType::Focus(direction) => focus::handle(&state, direction)?,
        KeybindingType::Swap(direction) => swap::handle(&state, direction)?,
        KeybindingType::Quit => sender.send(Event::Exit)?,
        KeybindingType::Split(direction) => split::handle(&state, direction)?,
        KeybindingType::ResetColumn => {
            let display = state.get_current_display_mut();

            if let Some(g) = display.get_focused_grid() {
                g.reset_column();
                g.draw_grid(display, &state.config);
            }
        }
        KeybindingType::ResetRow => {
            if let Some(g) = display.get_focused_grid() {
                g.reset_row();
                g.draw_grid(display, &state.config);
            }
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

                toggle_floating_mode::handle(&state)?;
            }
        }
    };

    Ok(())
}
