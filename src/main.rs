#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate strum_macros;

use winapi::um::winuser::SendMessageA;
use winapi::um::winuser::WM_DESTROY;
use winapi::um::winuser::GetForegroundWindow;
use winapi::um::winuser::SetWindowLongA;
use winapi::um::winuser::SetWindowPos;
use winapi::um::winuser::ShowWindow;
use winapi::um::winuser::SW_SHOW;
use winapi::um::winuser::GWL_STYLE;
use winapi::shared::windef::RECT;
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
mod app_bar;
mod win_event_handler;

use tile::Tile;
use tile_grid::TileGrid;
use config::Config;
use config::Keybinding;
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

unsafe fn on_quit(){
    if let Ok(mut grid) = GRID.lock() {
        grid
            .tiles
            .iter()
            .map(|tile| (tile.window.id, tile.window.original_style, tile.window.original_rect))
            .collect::<Vec<(i32, i32, RECT)>>() // collect because of borrow checker
            .iter()
            .for_each(|(id, style, original_rect)| {
                SetWindowLongA(*id as HWND, GWL_STYLE, *style);
                SetWindowPos(
                    *id as HWND, 
                    std::ptr::null_mut(), 
                    original_rect.left, 
                    original_rect.top, 
                    original_rect.right - original_rect.left, 
                    original_rect.bottom - original_rect.top, 
                    0
                );
                grid.close_tile_by_window_id(*id);
            });
        if CONFIG.remove_task_bar {
            ShowWindow(grid.taskbar_window as HWND, SW_SHOW);
        }
    }
    win_event_handler::unregister();
    std::process::exit(0);
}

fn main() {
    lazy_static::initialize(&CONFIG);
    lazy_static::initialize(&GRID);

    unsafe {

        app_bar::create();

        ctrlc::set_handler(|| {
            on_quit();
        });

        let mut hot_key_manager = HotKeyManager::new();

        for keybinding in CONFIG.keybindings.iter() {
            let (key, modifiers, callback): (&Key, &Vec<Modifier>, Box<dyn Fn()>) = match keybinding {
                Keybinding::Shell(key, modifiers, cmd) => (key, modifiers, Box::new(move || { 
                    Command::new("cmd").args(&["/C", &cmd]).spawn(); 
                })),
                Keybinding::CloseTile(key, modifiers) => (key, modifiers, Box::new(move || {   
                    if let Ok(mut grid) = GRID.lock() {
                        if let Some(id) = grid.focused_window_id {
                            SendMessageA(id as HWND, WM_DESTROY, 0, 0);
                            grid.close_tile_by_window_id(id);
                            grid.print_grid();
                        }
                    } 
                })),
                Keybinding::ToggleFloatingMode(key, modifiers) => (key, modifiers, Box::new(move || {   
                    let window_handle = GetForegroundWindow();
                    let mut maybe_tile: Option<Tile> = None;

                    if let Ok(grid) = GRID.lock() {
                        maybe_tile = grid.get_focused_tile().map(|x| x.clone());
                    } 

                    println!("test {} {}", maybe_tile.is_some(), window_handle as i32);

                    if let Some(tile) = maybe_tile {
                        let window_id = tile.window.id;
                        if window_id as HWND == window_handle {
                            SetWindowLongA(window_handle, GWL_STYLE, tile.window.original_style);
                            SetWindowPos(
                                window_id as HWND, 
                                std::ptr::null_mut(), 
                                tile.window.original_rect.left, 
                                tile.window.original_rect.top, 
                                tile.window.original_rect.right - tile.window.original_rect.left, 
                                tile.window.original_rect.bottom - tile.window.original_rect.top, 
                                0
                            );

                            if let Ok(mut grid) = GRID.lock() {
                                grid.close_tile_by_window_id(window_id);
                                grid.print_grid();
                            } 
                        }
                        else {
                            win_event_handler::split_window(window_handle as HWND);
                        }
                    } else {
                        win_event_handler::split_window(window_handle as HWND);
                    }
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
                        }
                        grid.print_grid();
                    }
                })),
                Keybinding::Quit(key, modifiers) => (key, modifiers, Box::new(move || {   
                    on_quit();
                })),
                Keybinding::Split(key, modifiers, direction) => (key, modifiers, Box::new(move || {   
                    if let Ok(mut grid) = GRID.lock() {
                        grid.set_focused_split_direction(direction.clone());
                    }
                })),
            };

            hot_key_manager.register_hot_key(
                *key, 
                modifiers.clone(), 
                callback
            );
        }

        win_event_handler::register();
        hot_key_manager.start()
    }
}
