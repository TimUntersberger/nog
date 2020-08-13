#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate strum_macros;

use config::Config;
use crossbeam_channel::select;
use display::Display;
use event::Event;
use event::EventChannel;
use hot_reload::update_config;
use lazy_static::lazy_static;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Mutex;
use tile_grid::TileGrid;
use winapi::shared::windef::HWND;
use workspace::{change_workspace, Workspace};

mod bar;
mod config;
mod direction;
mod display;
mod event;
mod event_handler;
mod hot_reload;
mod keybindings;
mod logging;
mod message_loop;
mod popup;
mod split_direction;
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
    pub static ref CONFIG: Mutex<Config> = Mutex::new(
        config::rhai::engine::parse_config()
            .map_err(|e| error!("{}", e))
            .expect("Failed to load config")
    );
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

    info!("Initializing bars");

    change_workspace(1, false).expect("Failed to change workspace to ID@1");

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
            bar::create::create()?;
        }

        info!("Registering windows event handler");
        win_event_handler::register()?;
    }

    info!("Listening for keybindings");
    keybindings::register()?;

    loop {
        select! {
            recv(receiver) -> maybe_msg => {
                let msg = maybe_msg.unwrap();
                let _ = match msg {
                    Event::Keybinding(kb) => event_handler::keybinding::handle(kb),
                    Event::RedrawAppBar => Ok(bar::redraw::redraw()),
                    Event::WinEvent(ev) => event_handler::winevent::handle(ev),
                    Event::Exit => {
                        tray::remove_icon(*tray::WINDOW.lock().unwrap() as HWND);
                        on_quit()?;
                        break;
                    },
                    Event::ReloadConfig => {
                        info!("Reloading Config");

                        bar::clear();

                        bar::empty_components();

                        update_config(config::rhai::engine::parse_config().expect("Failed to load config"))
                    }
                }.map_err(|e| {
                    error!("{}", e);
                });
            }
        }
    }

    Ok(())
}

fn main() {
    logging::setup().expect("Failed to setup logging");

    let panic = std::panic::catch_unwind(|| {
        info!("");

        #[cfg(not(debug_assertions))]
        update::start().expect("Failed to start update job");

        ctrlc::set_handler(|| {
            if let Err(e) = on_quit() {
                error!("Something happend when cleaning up. {}", e);
            }
        })
        .unwrap();

        display::init();
        popup::Popup::new("Test window".into(), 200, 100).create();

        // if let Err(e) = run() {
        //     error!("An error occured {:?}", e);
        //     if let Err(e) = on_quit() {
        //         error!("Something happend when cleaning up. {}", e);
        //     }
        // }
    });

    if let Err(err) = panic {
        if let Ok(msg) = err.downcast::<&'static str>() {
            error!("PANIC: {}", msg);
        } else {
            error!("PANIC: unknown");
        }
    }
}
