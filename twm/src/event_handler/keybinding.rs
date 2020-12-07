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
            state.move_workspace_to_monitor(monitor)?;
        }
        KeybindingType::CloseTile => state.close_window()?,
        KeybindingType::MinimizeTile => {
            state.minimize_window()?;
        }
        KeybindingType::MoveToWorkspace(id) => {
            state.move_window_to_workspace(id)?;
        }
        KeybindingType::ChangeWorkspace(id) => state.change_workspace(id, false),
        KeybindingType::ToggleFloatingMode => state.toggle_floating()?,
        KeybindingType::ToggleFullscreen => {
            state.toggle_fullscreen()?;
        }
        KeybindingType::ToggleMode(mode) => {
            state.toggle_mode(mode);
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
            state.reset_column()?;
        }
        KeybindingType::ResetRow => {
            state.reset_row()?;
        }
        KeybindingType::Callback(idx) => {
            drop(state);
            sender.send(Event::CallCallback(idx)).unwrap();
        }
        KeybindingType::IgnoreTile => {
            state.ignore_window()?;
        }
    };

    Ok(())
}
