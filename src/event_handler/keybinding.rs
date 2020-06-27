mod close_tile;
mod focus;
mod split;
mod swap;
mod toggle_floating_mode;
mod toggle_work_mode;

use crate::WORKSPACE_ID;
use crate::GRIDS;
use crate::change_workspace;
use crate::event::Event;
use crate::hot_key_manager::Keybinding;
use crate::hot_key_manager::KeybindingType;
use crate::CHANNEL;


use crate::WORK_MODE;
use winapi::um::processthreadsapi::CreateProcessA;
use winapi::um::processthreadsapi::PROCESS_INFORMATION;
use winapi::um::processthreadsapi::STARTUPINFOA;

use log::{error, info};


pub fn handle(kb: Keybinding) -> Result<(), Box<dyn std::error::Error>> {
    info!("Received keybinding of type {:?}", kb.typ);
    let sender = CHANNEL.sender.clone();
    if *WORK_MODE.lock().unwrap() {
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
            KeybindingType::CloseTile => close_tile::handle()?,
            KeybindingType::MoveToWorkspace(id) => {
                let mut grids = GRIDS.lock().unwrap();
                let grid = grids
                    .iter_mut()
                    .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
                    .unwrap();

                if let Some(window_id) = grid.focused_window_id {
                    if let Some(tile) = grid.close_tile_by_window_id(window_id) {
                        let grid = grids
                            .iter_mut()
                            .find(|g| g.id == id)
                            .unwrap();
                        grid.split(tile.window);
                        drop(grids);
                        change_workspace(id)?;
                    }
                }
            },
            KeybindingType::ChangeWorkspace(id) => change_workspace(id)?,
            KeybindingType::ToggleFloatingMode => toggle_floating_mode::handle()?,
            KeybindingType::ToggleWorkMode => toggle_work_mode::handle()?,
            KeybindingType::Focus(direction) => focus::handle(direction)?,
            KeybindingType::Swap(direction) => swap::handle(direction)?,
            KeybindingType::Quit => sender.send(Event::Exit)?,
            KeybindingType::Split(direction) => split::handle(direction)?,
        };
    } else if kb.typ == KeybindingType::ToggleWorkMode {
        toggle_work_mode::handle()?
    }

    Ok(())
}
