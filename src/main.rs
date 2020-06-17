#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate strum_macros;

use winapi::um::winuser::FindWindowA;
use winapi::shared::windef::HWND;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::process::Command;

mod tile_grid;
mod task_bar;
mod display;
mod hot_key_manager;
mod config;
mod tile;
mod window;
mod util;
mod app_bar;
mod win_event_handler;

use tile::Tile;
use tile_grid::TileGrid;
use window::Window;
use config::Config;
use config::Keybinding;
use hot_key_manager::HotKeyManager;
use hot_key_manager::Key;
use hot_key_manager::Modifier;

lazy_static! {
    pub static ref GRID: Mutex<TileGrid> = {
        let mut grid = TileGrid::new();

        unsafe {
            grid.height = display::HEIGHT;
            grid.width = display::WIDTH;
        }

        println!("Height: {} | width: {}", grid.height, grid.width);

        return Mutex::new(grid);
    };
    pub static ref CONFIG: Config = config::load().unwrap();
}

fn on_quit() -> Result<(), util::WinApiResultError> {
    if let Ok(mut grid) = GRID.lock() {
        for tile in grid.tiles.clone() {
            grid.close_tile_by_window_id(tile.window.id);
            tile.window.reset_style()?;
            tile.window.reset_pos()?;
        }
    }

    if CONFIG.remove_task_bar {
        task_bar::show();
    }
    win_event_handler::unregister()?;

    std::process::exit(0);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    lazy_static::initialize(&CONFIG);
    
    //TODO: if i remove this println the program doesn't start anymore? xd
    println!("test");
    task_bar::init();
    display::init();

    lazy_static::initialize(&GRID);

    if CONFIG.remove_task_bar {
        task_bar::hide();
    }

    if CONFIG.display_app_bar {
        app_bar::create()?;
    }

    ctrlc::set_handler(|| {
        on_quit().unwrap()
    })?;

    let mut hot_key_manager = HotKeyManager::new();

    for keybinding in CONFIG.keybindings.iter() {
        let (key, modifiers, callback): (&Key, &Vec<Modifier>, Box<dyn Fn() -> Result<(), Box<dyn std::error::Error>> + Send + Sync>) = match keybinding {
            Keybinding::Shell(key, modifiers, cmd) => (key, modifiers, Box::new(move || { 
                Command::new("cmd").args(&["/C", &cmd]).spawn()?;

                Ok(())
            })),
            Keybinding::CloseTile(key, modifiers) => (key, modifiers, Box::new(move || {   
                if let Ok(mut grid) = GRID.lock() {
                    if let Some(tile) = grid.get_focused_tile() {
                        grid.print_grid();
                        tile.window.send_destroy();
                        let id = tile.window.id; //need this variable because of borrow checker
                        grid.close_tile_by_window_id(id);
                    }
                }

                Ok(())
            })),
            Keybinding::ToggleFloatingMode(key, modifiers) => (key, modifiers, Box::new(move || {   
                let window_handle = Window::get_foreground_window()?;
                let mut maybe_tile: Option<Tile> = None;

                if let Ok(grid) = GRID.lock() {
                    maybe_tile = grid.get_focused_tile().map(|x| x.clone());
                } 

                println!("test {} {}", maybe_tile.is_some(), window_handle as i32);

                if let Some(tile) = maybe_tile {
                    if tile.window.id as HWND == window_handle {
                        tile.window.reset_style()?;
                        tile.window.reset_pos()?;

                        if let Ok(mut grid) = GRID.lock() {
                            grid.close_tile_by_window_id(tile.window.id);
                            grid.print_grid();
                        } 
                    }
                    else {
                        win_event_handler::split_window(window_handle as HWND)?;
                    }
                } else {
                    win_event_handler::split_window(window_handle as HWND)?;
                }

                Ok(())
            })),
            Keybinding::Focus(key, modifiers, direction) => (key, modifiers, Box::new(move || {   
                if let Ok(mut grid) = GRID.lock() {
                    match direction.as_str() {
                        "Left" => grid.focus_left(),
                        "Right" => grid.focus_right(),
                        "Up" => grid.focus_up(),
                        "Down" => grid.focus_down(),
                        x => { 
                            println!("invalid direction {} for focus keybinding", x);
                            panic!();
                        }
                    }?;
                    grid.print_grid();
                }

                Ok(())
            })),
            Keybinding::Quit(key, modifiers) => (key, modifiers, Box::new(move || {   
                on_quit()?;

                Ok(())
            })),
            Keybinding::Split(key, modifiers, direction) => (key, modifiers, Box::new(move || {   
                if let Ok(mut grid) = GRID.lock() {
                    grid.set_focused_split_direction(direction.clone());
                }

                Ok(())
            })),
        };

        hot_key_manager.register_hot_key(
            *key, 
            modifiers.clone(), 
            callback
        )?;
    }

    win_event_handler::register()?;
    hot_key_manager.start()?;

    Ok(())
}
