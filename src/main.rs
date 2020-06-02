#[macro_use]
extern crate num_derive;

use winapi::um::winuser::SendMessageA;
use winapi::um::winuser::WM_DESTROY;
use winapi::um::winuser::MSG;
use winapi::shared::windef::HWND;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::process::Command;

mod tile_grid;
mod hot_key_manager;
mod tile;
mod window;
mod util;
mod win_event_handler;

use tile_grid::TileGrid;
use tile_grid::SplitDirection;
use hot_key_manager::HotKeyManager;
use hot_key_manager::Key;
use hot_key_manager::Modifier;

lazy_static! {
    pub static ref GRID: Mutex<TileGrid> = {
        let mut grid = TileGrid::new();

        unsafe {
            grid.fetch_resolution();
        }

        println!("Height: {} | width: {}", grid.height, grid.width);

        return Mutex::new(grid);
    };
}


//TODO: Setup github repo for this project
//      Convert each todo into its own issue or at least its own card in a github project
//      Replace each todo with just its issue reference. For example:
//          TODO: test -> TODO(#20)
//      This will make it easier to find where to implement stuff without having to think about it again
//TODO: Support automatic hiding of the tasbar when starting and showing when closing the twm
//TODO: Maybe there is a way to support automatic customization of firefox
//      Specifically changing the titlebar style when the twm is open and then changing it again when the twm gets closed.
//      Might be able to support this for chromium too
//TODO: The height of the grid should respect whether the taskbar is shown or not.
//      Should probably just be a boolean config value
//      The default will be hidden I think
//TODO: Support a config file
//      It has to be possible to specify whether the twm should remove the titlebar of managed windows
//TODO: Support keybindings in the config file
//      Make it possible to execute an arbritary command with a shortcut using "cmd \C"
//TODO: Support a list of window names that don't get handled by the twm
//TODO: Support launch on startup
//TODO: Detect mouse events
//      The window id does currently not change when a user clicks on a different window fucking everything up.
//      This obviously has to be support
//      Maybe even detect ctrl tab? Not to sure about this one tbh
//TODO: Add a keybind for exiting the twm
//TODO: Add a keybind for editing the config 
//      Maybe this should be defined by every user themself since this keybind requires knowledge about installed editors and is overall really inconvenient to implement
//TODO: Add a keybind for toggling every window into a "floating mode"
//      A window in floating mode basically is still "managed" but not in a way where we interfer with anything, we just know that it exists
//      Maybe we should set every managed window into floating mode when using the exit keybind, maybe even when we detect a "close" event from the user (e.g. when the user uses alt+F4)
//TODO: Add a systemtray icon
//TODO: feature: Swap tile places
//TODO: Implement workspaces
//TODO: Handle edge cases for reordering the tiles after creation or deletion
//TODO: Look for a way to splitup TileGrid. It currently has way too much lines

fn main() {
    unsafe {
        let mut hot_key_manager = HotKeyManager::new();

        hot_key_manager.register_hot_key(
            Key::Enter, 
            vec![Modifier::Control, Modifier::Alt], 
            || { Command::new("cmd").args(&["/C", "wt"]).spawn(); }
        );

        hot_key_manager.register_hot_key(
            Key::P, 
            vec![Modifier::Control, Modifier::Alt], 
            || { Command::new("cmd").args(&["/C", "wt -p Windows PowerShell"]).spawn(); }
        );

        hot_key_manager.register_hot_key(
            Key::Q, 
            vec![Modifier::Control, Modifier::Alt], 
            || {   
                if let Ok(mut grid) = GRID.lock() {
                    if let Some(id) = grid.focused_window_id {
                        SendMessageA(id as HWND, WM_DESTROY, 0, 0);
                        grid.close_tile_by_window_id(id);
                        grid.print_grid();
                    }
                } 
            }
        );       

        hot_key_manager.register_hot_key(
            Key::H, 
            vec![Modifier::Control, Modifier::Alt], 
            || {   
                if let Ok(mut grid) = GRID.lock() {
                    grid.focus_left();
                    grid.print_grid();
                }
            }
        );

        hot_key_manager.register_hot_key(
            Key::J, 
            vec![Modifier::Control, Modifier::Alt], 
            || {   
                if let Ok(mut grid) = GRID.lock() {
                    grid.focus_down();
                    grid.print_grid();
                }
            }
        );

        hot_key_manager.register_hot_key(
            Key::K, 
            vec![Modifier::Control, Modifier::Alt], 
            || {   
                if let Ok(mut grid) = GRID.lock() {
                    grid.focus_up();
                    grid.print_grid();
                }
            }
        );

        hot_key_manager.register_hot_key(
            Key::L, 
            vec![Modifier::Control, Modifier::Alt], 
            || {   
                if let Ok(mut grid) = GRID.lock() {
                    grid.focus_right();
                    grid.print_grid();
                }
            }
        );

        hot_key_manager.register_hot_key(
            Key::Plus, 
            vec![Modifier::Control, Modifier::Alt], 
            || {   
                if let Ok(mut grid) = GRID.lock() {
                    grid.set_focused_split_direction(SplitDirection::Vertical);
                }
            }
        );

        hot_key_manager.register_hot_key(
            Key::Minus, 
            vec![Modifier::Control, Modifier::Alt], 
            || {   
                if let Ok(mut grid) = GRID.lock() {
                    grid.set_focused_split_direction(SplitDirection::Horizontal);
                }
            }
        );

        win_event_handler::register();
        hot_key_manager.start()
    }
}
