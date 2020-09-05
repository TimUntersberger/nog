#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate strum_macros;

use config::{rule::Rule, Config};
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
    pub static ref ADDITIONAL_RULES: Mutex<Vec<Rule>> = Mutex::new(Vec::new());
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
            tile.window.reset();
        }
    }

    Ok(())
}

pub fn with_current_grid<TFunction, TReturn>(f: TFunction) -> TReturn
where
    TFunction: Fn(&mut TileGrid) -> TReturn,
{
    with_grid_by_id(*WORKSPACE_ID.lock().unwrap(), f)
}

pub fn with_grid_by_id<TFunction, TReturn>(id: i32, f: TFunction) -> TReturn
where
    TFunction: Fn(&mut TileGrid) -> TReturn,
{
    let mut grids = GRIDS.lock().unwrap();
    let mut grid = grids.iter_mut().find(|g| g.id == id).unwrap();

    f(&mut grid)
}

fn on_quit() -> Result<(), util::WinApiResultError> {
    unmanage_everything()?;

    popup::cleanup();
    let remove_task_bar = {
        let config = CONFIG.lock().unwrap();
        config.remove_task_bar
    };

    if remove_task_bar {
        task_bar::show_taskbars();
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

    info!("Initializing popups");
    popup::init();

    for display in DISPLAYS.lock().unwrap().iter() {
        VISIBLE_WORKSPACES
            .lock()
            .unwrap()
            .insert(display.hmonitor, 0);
    }

    info!("Starting hot reloading of config");
    config::hot_reloading::start();

    startup::set_launch_on_startup(CONFIG.lock().unwrap().launch_on_startup)?;

    info!("Creating tray icon");
    tray::create()?;

    info!("Initializing workspaces");
    lazy_static::initialize(&WORKSPACES);

    if *WORK_MODE.lock().unwrap() {
        if CONFIG.lock().unwrap().remove_task_bar {
            info!("Hiding taskbar");
            task_bar::hide_taskbars();
        }

        if CONFIG.lock().unwrap().display_app_bar {
            bar::create::create()?;
        }

        info!("Registering windows event handler");
        win_event_handler::register()?;
    }

    info!("Initializing bars");

    change_workspace(1, false).expect("Failed to change workspace to ID@1");

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
