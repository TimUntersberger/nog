mod toggle_floating_mode;
mod toggle_work_mode;
mod close_tile;
mod focus;
mod swap;
mod split;

use crate::WORK_MODE;
use crate::CHANNEL;
use crate::change_workspace;
use crate::event::Event;
use crate::hot_key_manager::Keybinding;
use crate::hot_key_manager::KeybindingType;

use log::info;
use std::process::Command;

pub fn handle(kb: Keybinding) -> Result<(), Box<dyn std::error::Error>> {
    info!("Received keybinding of type {:?}", kb.typ);
    let sender = CHANNEL.sender.clone();
    
    if *WORK_MODE.lock().unwrap() {
        match kb.typ {
            KeybindingType::Shell(cmd) => {Command::new("cmd").args(&["/C", &cmd]).spawn()?;},
            KeybindingType::CloseTile => close_tile::handle()?,
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
