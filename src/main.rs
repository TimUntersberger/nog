#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate strum_macros;

use lazy_static::lazy_static;
use log::{debug, error, info};
use std::process::Command;
use std::sync::Mutex;
use winapi::shared::windef::HWND;

mod app_bar;
mod config;
mod display;
mod hot_key_manager;
mod task_bar;
mod tile;
mod tile_grid;
mod util;
mod win_event_handler;
mod window;
mod workspace;

use config::Config;
use config::Keybinding;
use display::Display;
use hot_key_manager::HotKeyManager;
use hot_key_manager::Key;
use hot_key_manager::Modifier;
use tile_grid::TileGrid;
use window::Window;
use workspace::Workspace;

lazy_static! {
    pub static ref CONFIG: Config = config::load().unwrap();
    pub static ref DISPLAY: Mutex<Display> = {
        let mut display = Display::default();
        display.init();
        return Mutex::new(display);
    };
    pub static ref GRIDS: Mutex<Vec<TileGrid>> = {
        return Mutex::new(
            (1..10)
                .map(|i| {
                    let mut grid = TileGrid::new(i);

                    grid.height = DISPLAY.lock().unwrap().height - CONFIG.margin * 2;
                    grid.width = DISPLAY.lock().unwrap().width - CONFIG.margin * 2;

                    grid
                })
                .collect::<Vec<TileGrid>>(),
        );
    };
    pub static ref WORKSPACES: Mutex<Vec<Workspace>> = {
        return Mutex::new(
            (1..10)
                .map(|i| Workspace::new(i))
                .collect::<Vec<Workspace>>(),
        );
    };
    pub static ref WORKSPACE_ID: Mutex<i32> = Mutex::new(1);
}

fn on_quit() -> Result<(), util::WinApiResultError> {
    let mut grids = GRIDS.lock().unwrap();

    for grid in grids.iter_mut() {
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

fn draw_workspaces() {
    let id = *WORKSPACE_ID.lock().unwrap();

    app_bar::clear();

    GRIDS
        .lock()
        .unwrap()
        .iter()
        .filter(|t| t.tiles.len() > 0 || t.id == id)
        .enumerate()
        .for_each(|(i, t)| {
            app_bar::draw_workspace(i as i32, t.id, t.id == id);
        });
}

pub fn change_workspace(id: i32) -> Result<(), util::WinApiResultError> {
    let mut grids = GRIDS.lock().unwrap();
    let mut gid = WORKSPACE_ID.lock().unwrap();

    if *gid == id {
        debug!("Workspace is already selected");
        return Ok(());
    }

    let old_id = *gid;

    *gid = id;

    debug!("Showing the next workspace");
    grids.iter_mut().find(|g| g.id == *gid).unwrap().show();

    //without this delay there is a slight flickering of the background
    std::thread::sleep(std::time::Duration::from_millis(5));

    debug!("Hiding the current workspace");
    grids.iter_mut().find(|g| g.id == old_id).unwrap().hide();

    drop(grids);
    drop(gid);

    draw_workspaces();

    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing config");
    lazy_static::initialize(&CONFIG);

    info!("Initializing taskbar");
    task_bar::init();

    if CONFIG.remove_task_bar {
        info!("Hiding taskbar");
        task_bar::hide();
    }

    info!("Initializing display");
    lazy_static::initialize(&DISPLAY);

    if CONFIG.display_app_bar {
        info!("Creating appbar");
        app_bar::create(&*DISPLAY.lock().unwrap())?;
        app_bar::draw_workspace(0, 1, true)?;
    }

    info!("Initializing workspaces");
    lazy_static::initialize(&WORKSPACES);

    let mut hot_key_manager = HotKeyManager::new();

    for (i, keybinding) in CONFIG.keybindings.iter().enumerate() {
        let (key_type, key, modifier, callback): (
            &Keybinding,
            &Key,
            &Modifier,
            Box<dyn Fn() -> Result<(), Box<dyn std::error::Error>> + Send + Sync>,
        ) = match keybinding {
            Keybinding::Shell(key, modifier, cmd) => (
                keybinding,
                key,
                modifier,
                Box::new(move || {
                    info!("Received hotkey of type Shell");
                    Command::new("cmd").args(&["/C", &cmd]).spawn()?;

                    Ok(())
                }),
            ),
            Keybinding::CloseTile(key, modifier) => (
                keybinding,
                key,
                modifier,
                Box::new(move || {
                    info!("Received hotkey of type CloseTile");
                    let mut grids = GRIDS.lock().unwrap();
                    let grid = grids
                        .iter_mut()
                        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
                        .unwrap();

                    if let Some(tile) = grid.get_focused_tile() {
                        tile.window.send_close();
                        let id = tile.window.id; //need this variable because of borrow checker
                        grid.close_tile_by_window_id(id);
                        grid.draw_grid();
                    }

                    Ok(())
                }),
            ),
            Keybinding::ChangeWorkspace(key, modifier, id) => (
                keybinding,
                key,
                modifier,
                Box::new(move || {
                    info!("Received hotkey of type ChangeWorkspace");

                    change_workspace(*id)?;

                    Ok(())
                }),
            ),
            Keybinding::ToggleFloatingMode(key, modifier) => (
                keybinding,
                key,
                modifier,
                Box::new(move || {
                    info!("Received hotkey of type ToggleFloatingMode");
                    let window_handle = Window::get_foreground_window()?;

                    if let Ok(mut grids) = GRIDS.lock() {
                        if let Ok(gid) = WORKSPACE_ID.lock() {
                            let temp = grids
                                .iter_mut()
                                .find(|g| g.id == *gid)
                                .unwrap()
                                .get_focused_tile()
                                .map(|x| x.clone());

                            if let Some(tile) = temp {
                                if tile.window.id as HWND == window_handle {
                                    debug!(
                                        "Reseting window '{}' | {}",
                                        tile.window.title, tile.window.id
                                    );
                                    tile.window.reset_style()?;
                                    tile.window.reset_pos()?;

                                    let grid = grids.iter_mut().find(|g| g.id == *gid).unwrap();

                                    debug!(
                                        "Unmanaging window '{}' | {}",
                                        tile.window.title, tile.window.id
                                    );
                                    grid.close_tile_by_window_id(tile.window.id);
                                    grid.draw_grid();
                                } else {
                                    //the else block below has to have the same code
                                    //TODO: refactor to combine both of these else blocks
                                    for grid in grids.iter() {
                                        if grid.get_tile_by_id(window_handle as i32).is_some() {
                                            debug!("Window is in a different workspace. Aborting.");
                                            return Ok(());
                                        }
                                    }

                                    drop(grids);
                                    drop(gid);
                                    win_event_handler::split_window(window_handle as HWND)?;
                                }
                            } else {
                                for grid in grids.iter() {
                                    if grid.get_tile_by_id(window_handle as i32).is_some() {
                                        debug!("Window is in a different workspace. Aborting.");
                                        return Ok(());
                                    }
                                }

                                drop(grids);
                                drop(gid);
                                win_event_handler::split_window(window_handle as HWND)?;
                            }
                        }
                    }

                    Ok(())
                }),
            ),
            Keybinding::Focus(key, modifier, direction) => (
                keybinding,
                key,
                modifier,
                Box::new(move || {
                    info!("Received hotkey of type Focus");
                    let mut grids = GRIDS.lock().unwrap();
                    let grid = grids
                        .iter_mut()
                        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
                        .unwrap();

                    grid.focus(*direction)?;
                    grid.draw_grid();

                    Ok(())
                }),
            ),
            Keybinding::Swap(key, modifier, direction) => (
                keybinding,
                key,
                modifier,
                Box::new(move || {
                    info!("Received hotkey of type Swap");
                    let mut grids = GRIDS.lock().unwrap();

                    let grid = grids
                        .iter_mut()
                        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
                        .unwrap();

                    grid.swap(*direction)?;
                    grid.draw_grid();

                    Ok(())
                }),
            ),
            Keybinding::Quit(key, modifier) => (
                keybinding,
                key,
                modifier,
                Box::new(move || {
                    info!("Received hotkey of type Quit");
                    on_quit()?;

                    Ok(())
                }),
            ),
            Keybinding::Split(key, modifier, direction) => (
                keybinding,
                key,
                modifier,
                Box::new(move || {
                    info!("Received hotkey of type Split");
                    let mut grids = GRIDS.lock().unwrap();
                    grids
                        .iter_mut()
                        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
                        .unwrap()
                        .set_focused_split_direction(direction.clone());

                    Ok(())
                }),
            ),
        };

        info!(
            "Registering Keybinding({}+{}, {}) {}",
            format!("{:?}", modifier).replace(" | ", "+"),
            *key,
            *key_type,
            *key as i32
        );
        hot_key_manager.register_hot_key(*key, *modifier, callback, i as i32)?;
    }

    info!("Registering windows event handler");
    win_event_handler::register()?;

    info!("Starting hot key manager");
    hot_key_manager.start()?;

    Ok(())
}

fn main() {
    env_logger::init();

    //Handle unexpected panics
    std::panic::catch_unwind(|| {
        ctrlc::set_handler(|| {
            if let Err(e) = on_quit() {
                error!("Something happend when cleaning up. {}", e);
            }
        })
        .unwrap();

        run().unwrap();
    })
    .map_err(|e| {
        error!("An unexpected error occured {:?}", e);
        if let Err(e) = on_quit() {
            error!("Something happend when cleaning up. {}", e);
        }
    })
    .unwrap();
}
