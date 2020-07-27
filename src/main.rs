#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate strum_macros;

use crate::display::get_display_by_hmonitor;
use crate::display::get_display_by_idx;
use app_bar::RedrawAppBarReason;
use config::Config;
use crossbeam_channel::select;
use display::Display;
use event::Event;
use event::EventChannel;
use lazy_static::lazy_static;
use log::{debug, error, info};
use std::collections::HashMap;
use std::sync::Mutex;
use tile_grid::TileGrid;
use winapi::shared::windef::HWND;
use workspace::Workspace;

mod app_bar;
mod config;
mod display;
mod event;
mod event_handler;
mod hot_key_manager;
mod logging;
mod startup;
mod task_bar;
mod tile;
mod tile_grid;
mod tray;
mod update;
mod util;
mod win_event_handler;
mod window;
mod workspace;

lazy_static! {
    pub static ref WORK_MODE: Mutex<bool> = Mutex::new(CONFIG.lock().unwrap().work_mode);
    pub static ref CONFIG: Mutex<Config> =
        Mutex::new(config::load().expect("Failed to load config"));
    pub static ref DISPLAYS: Mutex<Vec<Display>> = Mutex::new(Vec::new());
    pub static ref CHANNEL: EventChannel = EventChannel::default();
    pub static ref GRIDS: Mutex<Vec<TileGrid>> =
        Mutex::new((1..11).map(TileGrid::new).collect::<Vec<TileGrid>>());
    pub static ref WORKSPACES: Mutex<Vec<Workspace>> =
        Mutex::new((1..11).map(Workspace::new).collect::<Vec<Workspace>>());
    pub static ref VISIBLE_WORKSPACES: Mutex<HashMap<i32, i32>> = Mutex::new(HashMap::new());
    pub static ref WORKSPACE_ID: Mutex<i32> = Mutex::new(1);
}

fn unmanage_everything() -> Result<(), util::WinApiResultError> {
    let mut grids = GRIDS.lock().unwrap();

    for grid in grids.iter_mut() {
        for tile in &mut grid.tiles.clone() {
            grid.close_tile_by_window_id(tile.window.id);
            tile.window.reset()?;
        }
    }

    Ok(())
}

fn on_quit() -> Result<(), util::WinApiResultError> {
    unmanage_everything()?;

    let config = CONFIG.lock().unwrap();

    if config.remove_task_bar {
        task_bar::show();
    }

    win_event_handler::unregister()?;

    std::process::exit(0);
}

pub fn is_visible_workspace(id: i32) -> bool {
    VISIBLE_WORKSPACES
        .lock()
        .unwrap()
        .values()
        .any(|v| *v == id)
}

pub fn change_workspace(id: i32) -> Result<(), util::WinApiResultError> {
    let mut grids = GRIDS.lock().unwrap();

    let workspace_settings = CONFIG.lock().unwrap().workspace_settings.clone();

    let (new_grid_idx, mut new_grid) = grids
        .iter_mut()
        .enumerate()
        .find(|(_, g)| g.id == id)
        .map(|(i, g)| (i, g.clone()))
        .unwrap();

    if let Some(setting) = workspace_settings.iter().find(|s| s.id == id) {
        new_grid.display = get_display_by_idx(setting.monitor);
    }

    let mut visible_workspaces = VISIBLE_WORKSPACES.lock().unwrap();

    debug!("Drawing the workspace");
    new_grid.draw_grid();
    debug!("Showing the workspace");
    new_grid.show();

    if let Some(id) = visible_workspaces.insert(new_grid.display.hmonitor, new_grid.id) {
        if new_grid.id != id {
            if let Some(grid) = grids.iter().find(|g| g.id == id) {
                debug!("Hiding the current workspace");
                grid.hide();
            } else {
                debug!("Workspace is already visible");
            }
        }
    }

    debug!("Updating workspace id of monitor");
    grids.remove(new_grid_idx);
    grids.insert(new_grid_idx, new_grid);

    *WORKSPACE_ID.lock().unwrap() = id;

    debug!("Sending redraw-app-bar event");
    CHANNEL
        .sender
        .clone()
        .send(Event::RedrawAppBar(RedrawAppBarReason::Workspace))
        .expect("Failed to send redraw-app-bar event");

    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let receiver = CHANNEL.receiver.clone();

    info!("Initializing config");
    lazy_static::initialize(&CONFIG);

    info!("Initializing displays");
    display::init();

    for display in DISPLAYS.lock().unwrap().iter() {
        VISIBLE_WORKSPACES
            .lock()
            .unwrap()
            .insert(display.hmonitor, 0);
    }

    change_workspace(1).expect("Failed to change workspace to ID@1");

    info!("Starting hot reloading of config");
    config::hot_reloading::start();

    startup::set_launch_on_startup(CONFIG.lock().unwrap().launch_on_startup)?;

    info!("Initializing taskbar");
    task_bar::init();

    info!("Creating tray icon");
    tray::create()?;

    info!("Initializing workspaces");
    lazy_static::initialize(&WORKSPACES);

    if *WORK_MODE.lock().unwrap() {
        if CONFIG.lock().unwrap().remove_task_bar {
            info!("Hiding taskbar");
            task_bar::hide();
        }

        if CONFIG.lock().unwrap().display_app_bar {
            app_bar::create()?;
        }

        info!("Registering windows event handler");
        win_event_handler::register()?;
    }

    info!("Starting hot key manager");
    hot_key_manager::register()?;

    loop {
        select! {
            recv(receiver) -> maybe_msg => {
                let msg = maybe_msg.unwrap();
                match msg {
                    Event::Keybinding(kb) => event_handler::keybinding::handle(kb)?,
                    Event::RedrawAppBar(reason) => app_bar::redraw(reason),
                    Event::WinEvent(ev) => event_handler::winevent::handle(ev)?,
                    Event::Exit => {
                        tray::remove_icon(*tray::WINDOW.lock().unwrap() as HWND);
                        on_quit()?;
                        break;
                    },
                    Event::ReloadConfig => {
                        info!("Reloading Config");

                        update_config(config::load().expect("Failed to load config"))?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn update_config(new_config: Config) -> Result<(), Box<dyn std::error::Error>> {
    hot_key_manager::unregister();

    let config = CONFIG.lock().unwrap().clone();
    let work_mode = *WORK_MODE.lock().unwrap();
    let mut draw_app_bar = false;

    if work_mode {
        if config.display_app_bar && new_config.display_app_bar {
            if config.app_bar_bg != new_config.app_bar_bg
            || config.app_bar_font != new_config.app_bar_font
            || config.app_bar_font_size != new_config.app_bar_font_size
            || config.app_bar_height != new_config.app_bar_height
            || config.light_theme != new_config.light_theme {
                app_bar::close();
                draw_app_bar = true;
            }
        }
        else if config.display_app_bar && !new_config.display_app_bar {
            app_bar::close();

            for d in DISPLAYS.lock().unwrap().iter_mut() {
                d.bottom += config.app_bar_height;
            }

            for grid in GRIDS.lock().unwrap().iter_mut() {
                grid.display = get_display_by_hmonitor(grid.display.hmonitor);
            }

        } else if !config.display_app_bar && new_config.display_app_bar {
            draw_app_bar = true;

            for d in DISPLAYS.lock().unwrap().iter_mut() {
                d.bottom -= config.app_bar_height;
            }

            for grid in GRIDS.lock().unwrap().iter_mut() {
                grid.display = get_display_by_hmonitor(grid.display.hmonitor);
            }
        }
        if config.remove_task_bar && !new_config.remove_task_bar {
            task_bar::show();
        } else if !config.remove_task_bar && new_config.remove_task_bar {
            task_bar::hide();
        }
    }

    if config.remove_title_bar && !new_config.remove_title_bar {
        let mut grids = GRIDS.lock().unwrap();

        for grid in grids.iter_mut() {
            for tile in &mut grid.tiles {
                tile.window.reset_style()?;
                tile.window.update_style();
            }
        }
    } else if !config.remove_title_bar && new_config.remove_title_bar {
        let mut grids = GRIDS.lock().unwrap();

        for grid in grids.iter_mut() {
            for tile in &mut grid.tiles {
                tile.window.remove_title_bar();
                tile.window.update_style();
            }
        }
    }

    if config.launch_on_startup != new_config.launch_on_startup {
        startup::set_launch_on_startup(new_config.launch_on_startup)?;
    }

    *CONFIG.lock().unwrap() = new_config;

    if draw_app_bar {
        app_bar::create()?;
        app_bar::show();
    }

    hot_key_manager::register()?;

    let mut grids = GRIDS.lock().unwrap();
    let grid = grids
        .iter_mut()
        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
        .unwrap();

    grid.draw_grid();

    Ok(())
}

fn main() {
    logging::setup().expect("Failed to setup logging");

    let panic = std::panic::catch_unwind(|| {
        info!("");

        update::update().expect("Failed to update the program");

        ctrlc::set_handler(|| {
            if let Err(e) = on_quit() {
                error!("Something happend when cleaning up. {}", e);
            }
        })
        .unwrap();

        if let Err(e) = run() {
            error!("An error occured {:?}", e);
            if let Err(e) = on_quit() {
                error!("Something happend when cleaning up. {}", e);
            }
        }
    });

    if let Err(err) = panic {
        if let Ok(msg) = err.downcast::<&'static str>() {
            error!("PANIC: {}", msg);
        } else {
            error!("PANIC: unknown");
        }
    }
}
