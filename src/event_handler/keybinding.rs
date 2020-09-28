use std::sync::Arc;

use crate::{
    config::{rhai::engine, rule::Rule},
    display::with_display_by_idx,
    event::Event,
    hot_reload::update_config,
    keybindings::{self, keybinding::Keybinding, keybinding_type::KeybindingType},
    system::api,
    with_current_grid, with_grid_by_id,
    workspace::change_workspace,
    ADDITIONAL_RULES, CHANNEL, CONFIG, VISIBLE_WORKSPACES,
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
    kb_manager: Arc<Mutex<KbManager>>,
    kb: Keybinding,
) -> Result<(), Box<dyn std::error::Error>> {
    if let KeybindingType::MoveWorkspaceToMonitor(_) = kb.typ {
        if !CONFIG.lock().multi_monitor {
            return Ok(());
        }
    }

    info!("Received keybinding of type {:?}", kb.typ);
    let sender = CHANNEL.sender.clone();
    match kb.typ {
        KeybindingType::Launch(cmd) => {
            api::launch_program(cmd)?;
        }
        KeybindingType::MoveWorkspaceToMonitor(monitor) => {
            let (grid_id, grid_old_monitor) = with_current_grid(|grid| {
                let old_id = grid.display.id;

                grid.display = with_display_by_idx(monitor, |d| d.unwrap().clone());

                (grid.id, old_id)
            });

            VISIBLE_WORKSPACES.lock().insert(grid_old_monitor, 0);

            change_workspace(grid_id, true);
        }
        KeybindingType::CloseTile => close_tile::handle()?,
        KeybindingType::MinimizeTile => {
            with_current_grid(|grid| {
                if let Some(tile) = grid.get_focused_tile_mut() {
                    let id = tile.window.id;

                    tile.window.minimize();
                    tile.window.cleanup();

                    grid.close_tile_by_window_id(id);
                }
            });
        }
        KeybindingType::MoveToWorkspace(id) => {
            let maybe_tile = with_current_grid(|grid| {
                grid.focused_window_id
                    .and_then(|id| grid.close_tile_by_window_id(id))
            });

            if let Some(tile) = maybe_tile {
                with_grid_by_id(id, |grid| {
                    grid.split(tile.window.clone());
                });
                change_workspace(id, false);
            }
        }
        KeybindingType::ChangeWorkspace(id) => change_workspace(id, false),
        KeybindingType::ToggleFloatingMode => toggle_floating_mode::handle()?,
        KeybindingType::ToggleFullscreen => {
            with_current_grid(|grid| {
                if !grid.tiles.is_empty() {
                    grid.fullscreen = !grid.fullscreen;

                    grid.draw_grid();
                }
            });
        }
        KeybindingType::ToggleMode(mode) => {
            if kb_manager.lock().get_mode() == Some(mode.clone()) {
                info!("Disabling {} mode", mode);
                kb_manager.lock().leave_mode();
            } else {
                info!("Enabling {} mode", mode);
                kb_manager.lock().enter_mode(&mode);
            }
        }
        KeybindingType::ToggleWorkMode => toggle_work_mode::handle(kb_manager)?,
        KeybindingType::IncrementConfig(field, value) => {
            let mut current_config = CONFIG.lock().clone();
            current_config.increment_field(&field, value);
            update_config(kb_manager, current_config)?;
        }
        KeybindingType::DecrementConfig(field, value) => {
            let mut current_config = CONFIG.lock().clone();
            current_config.decrement_field(&field, value);
            update_config(kb_manager, current_config)?;
        }
        KeybindingType::ToggleConfig(field) => {
            let mut current_config = CONFIG.lock().clone();
            current_config.toggle_field(&field);
            update_config(kb_manager, current_config)?;
        }
        KeybindingType::Resize(direction, amount) => resize::handle(direction, amount)?,
        KeybindingType::Focus(direction) => focus::handle(direction)?,
        KeybindingType::Swap(direction) => swap::handle(direction)?,
        KeybindingType::Quit => sender.send(Event::Exit)?,
        KeybindingType::Split(direction) => split::handle(direction)?,
        KeybindingType::ResetColumn => {
            with_current_grid(|grid| {
                if let Some(modification) = grid
                    .get_focused_tile()
                    .and_then(|t| t.column)
                    .and_then(|c| grid.column_modifications.get_mut(&c))
                {
                    modification.0 = 0;
                    modification.1 = 0;
                }

                grid.draw_grid();
            });
        }
        KeybindingType::ResetRow => {
            with_current_grid(|grid| {
                if let Some(modification) = grid
                    .get_focused_tile()
                    .and_then(|t| t.row)
                    .and_then(|c| grid.row_modifications.get_mut(&c))
                {
                    modification.0 = 0;
                    modification.1 = 0;
                }

                grid.draw_grid();
            });
        }
        KeybindingType::Callback(idx) => engine::call(idx),
        KeybindingType::IgnoreTile => {
            with_current_grid(|grid| {
                if let Some(tile) = grid.get_focused_tile() {
                    let process_name = tile.window.get_process_name();
                    let mut rules = ADDITIONAL_RULES.lock();
                    let mut rule = Rule::default();
                    let pattern = format!("^{}$", process_name);

                    debug!("Adding rule with pattern {}", pattern);

                    rule.pattern = regex::Regex::new(&pattern).expect("Failed to build regex");
                    rule.manage = false;

                    rules.push(rule);
                }
            });
            toggle_floating_mode::handle()?;
        }
    };

    Ok(())
}
