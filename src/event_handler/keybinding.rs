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
    state: &mut AppState,
    kb_manager: Arc<Mutex<KbManager>>,
    kb: Keybinding,
) -> Result<(), Box<dyn std::error::Error>> {
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
            let old_grid = state.get_current_grid();
            let new_grid = old_grid.clone();
            new_grid.display = state
                .get_display_by_idx(monitor)
                .expect("Failed to find display")
                .clone();

            state.visible_workspaces.insert(old_grid.display.id, 0);
            state.change_workspace(new_grid.id, true);

            // change_workspace(new_grid.id, true);
        }
        KeybindingType::CloseTile => close_tile::handle()?,
        KeybindingType::MinimizeTile => {
            let grid = state.get_current_grid();
            if let Some(tile) = grid.get_focused_tile_mut() {
                let id = tile.window.id;

                tile.window.minimize();
                tile.window.cleanup();

                grid.close_tile_by_window_id(id);
            }
        }
        KeybindingType::MoveToWorkspace(id) => {
            let grid = state.get_current_grid();
            grid.focused_window_id
                .and_then(|id| grid.close_tile_by_window_id(id))
                .map(|tile| {
                    state.get_grid_by_id(id).split(tile.window);
                    state.change_workspace(id, false);
                });
        }
        KeybindingType::ChangeWorkspace(id) => state.change_workspace(id, false),
        KeybindingType::ToggleFloatingMode => toggle_floating_mode::handle()?,
        KeybindingType::ToggleFullscreen => state.get_current_grid().toggle_fullscreen(),
        KeybindingType::ToggleMode(mode) => {
            if kb_manager.lock().get_mode() == Some(mode.clone()) {
                info!("Disabling {} mode", mode);
                kb_manager.lock().leave_mode();
            } else {
                info!("Enabling {} mode", mode);
                kb_manager.lock().enter_mode(&mode);
            }
        }
        KeybindingType::ToggleWorkMode => toggle_work_mode::handle(state, kb_manager)?,
        KeybindingType::IncrementConfig(field, value) => {
            update_config(
                state,
                kb_manager,
                state.config.increment_field(&field, value),
            )?;
        }
        KeybindingType::DecrementConfig(field, value) => {
            update_config(
                state,
                kb_manager,
                state.config.decrement_field(&field, value),
            )?;
        }
        KeybindingType::ToggleConfig(field) => {
            update_config(state, kb_manager, state.config.toggle_field(&field))?;
        }
        KeybindingType::Resize(direction, amount) => resize::handle(direction, amount)?,
        KeybindingType::Focus(direction) => focus::handle(direction)?,
        KeybindingType::Swap(direction) => swap::handle(direction)?,
        KeybindingType::Quit => sender.send(Event::Exit)?,
        KeybindingType::Split(direction) => split::handle(direction)?,
        KeybindingType::ResetColumn => {
            let grid = state.get_current_grid();
            grid.get_focused_tile()
                .and_then(|t| t.column)
                .and_then(|c| grid.column_modifications.get_mut(&c))
                .map(|m| {
                    m.0 = 0;
                    m.1 = 0;
                    grid.draw_grid();
                });
        }
        KeybindingType::ResetRow => {
            let grid = state.get_current_grid();
            grid.get_focused_tile()
                .and_then(|t| t.row)
                .and_then(|c| grid.row_modifications.get_mut(&c))
                .map(|m| {
                    m.0 = 0;
                    m.1 = 0;
                    grid.draw_grid();
                });
        }
        KeybindingType::Callback(idx) => engine::call(idx),
        KeybindingType::IgnoreTile => {
            if let Some(tile) = state.get_current_grid().get_focused_tile() {
                let mut rule = Rule::default();

                let process_name = tile.window.get_process_name();
                let pattern = format!("^{}$", process_name);

                debug!("Adding rule with pattern {}", pattern);

                rule.pattern = regex::Regex::new(&pattern).expect("Failed to build regex");
                rule.manage = false;

                state.additonal_rules.push(rule);

                toggle_floating_mode::handle()?;
            }
        }
    };

    Ok(())
}
