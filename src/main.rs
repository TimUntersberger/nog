#[macro_use]
extern crate num_derive;

use winapi::um::winuser::SendMessageA;
use winapi::um::winuser::WM_DESTROY;
use winapi::shared::windef::HWND;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::process::Command;

mod tile_grid;
mod hot_key_manager;
mod config;
mod tile;
mod window;
mod util;
mod win_event_handler;

use tile_grid::TileGrid;
use tile_grid::SplitDirection;
use config::Config;
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
    pub static ref CONFIG: Config = config::load().unwrap();
}

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
