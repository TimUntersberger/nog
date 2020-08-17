use crate::{
    display::get_display_by_idx,
    event::Event,
    hot_reload::update_config,
    keybindings::{self, keybinding::Keybinding, keybinding_type::KeybindingType},
    with_current_grid, with_grid_by_id,
    workspace::change_workspace,
    CHANNEL, CONFIG, VISIBLE_WORKSPACES, config::rhai::engine,
};
use log::{error, info};
use winapi::um::processthreadsapi::{CreateProcessA, PROCESS_INFORMATION, STARTUPINFOA};

mod close_tile;
mod focus;
mod resize;
mod split;
mod swap;
mod toggle_floating_mode;
pub mod toggle_work_mode;

pub fn handle(kb: Keybinding) -> Result<(), Box<dyn std::error::Error>> {
    if let KeybindingType::MoveWorkspaceToMonitor(_) = kb.typ {
        if !CONFIG.lock().unwrap().multi_monitor {
            return Ok(());
        }
    }

    info!("Received keybinding of type {:?}", kb.typ);
    let sender = CHANNEL.sender.clone();
    match kb.typ {
        KeybindingType::Launch(cmd) => {
            let mut si = STARTUPINFOA::default();
            let mut pi = PROCESS_INFORMATION::default();
            let mut cmd_bytes: Vec<u8> = cmd.bytes().chain(std::iter::once(0)).collect();

            unsafe {
                let x = CreateProcessA(
                    std::ptr::null_mut(),
                    cmd_bytes.as_mut_ptr() as *mut i8,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    0,
                    0,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut si,
                    &mut pi,
                );

                if x != 1 {
                    error!(
                        "Error launching program: {}",
                        winapi::um::errhandlingapi::GetLastError()
                    );
                }
            }
        }
        KeybindingType::MoveWorkspaceToMonitor(monitor) => {
            let (grid_id, grid_old_monitor) = with_current_grid(|grid| {
                let hmonitor = grid.display.hmonitor;

                grid.display = get_display_by_idx(monitor);

                (grid.id, hmonitor)
            });

            VISIBLE_WORKSPACES
                .lock()
                .unwrap()
                .insert(grid_old_monitor, 0);

            change_workspace(grid_id, true)
                .expect("Failed to change workspace after moving workspace to different monitor");
        }
        KeybindingType::CloseTile => close_tile::handle()?,
        KeybindingType::MinimizeTile => {
            with_current_grid(|grid| {
                if let Some(tile) = grid.get_focused_tile_mut() {
                    let id = tile.window.id;

                    tile.window.minimize();
                    tile.window.reset();

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
                change_workspace(id, false)?;
            }
        }
        KeybindingType::ChangeWorkspace(id) => change_workspace(id, false)?,
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
            if keybindings::enable_mode(&mode) {
                info!("Enabling {} mode", mode);
            } else {
                keybindings::disable_mode();
                info!("Disabling {} mode", mode);
            }
        }
        KeybindingType::ToggleWorkMode => toggle_work_mode::handle()?,
        KeybindingType::IncrementConfig(field, value) => {
            let mut current_config = CONFIG.lock().unwrap().clone();
            current_config.increment_field(&field, value);
            update_config(current_config)?;
        }
        KeybindingType::DecrementConfig(field, value) => {
            let mut current_config = CONFIG.lock().unwrap().clone();
            current_config.decrement_field(&field, value);
            update_config(current_config)?;
        }
        KeybindingType::ToggleConfig(field) => {
            let mut current_config = CONFIG.lock().unwrap().clone();
            current_config.toggle_field(&field);
            update_config(current_config)?;
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
        KeybindingType::Callback(fn_name) => engine::call(&fn_name),
    };

    Ok(())
}
