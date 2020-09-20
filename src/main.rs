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
use event_handler::keybinding::toggle_work_mode;
use hot_reload::update_config;
use lazy_static::lazy_static;
use log::debug;
use log::{error, info};
use parking_lot::{deadlock, Mutex};
use std::collections::HashMap;
use std::{thread, time::Duration};
use system::{DisplayId, SystemResult, WinEventListener};
use tile_grid::TileGrid;
use winapi::shared::windef::HWND;
use workspace::Workspace;

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
mod system;
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
    pub static ref WORK_MODE: Mutex<bool> = Mutex::new(CONFIG.lock().work_mode);
    pub static ref CONFIG: Mutex<Config> = Mutex::new(
        config::rhai::engine::parse_config()
            .map_err(|e| error!("{}", e))
            .expect("Failed to load config")
    );
    pub static ref DISPLAYS: Mutex<Vec<Display>> = Mutex::new(Vec::new());
    pub static ref CHANNEL: EventChannel = EventChannel::default();
    pub static ref ADDITIONAL_RULES: Mutex<Vec<Rule>> = Mutex::new(Vec::new());
    pub static ref WIN_EVENT_LISTENER: WinEventListener = WinEventListener::default();
    pub static ref GRIDS: Mutex<Vec<TileGrid>> =
        Mutex::new((1..11).map(TileGrid::new).collect::<Vec<TileGrid>>());
    pub static ref WORKSPACES: Mutex<Vec<Workspace>> =
        Mutex::new((1..11).map(Workspace::new).collect::<Vec<Workspace>>());
    pub static ref VISIBLE_WORKSPACES: Mutex<HashMap<DisplayId, i32>> = Mutex::new(HashMap::new());
    pub static ref WORKSPACE_ID: Mutex<i32> = Mutex::new(1);
}

fn unmanage_everything() -> SystemResult {
    let mut grids = GRIDS.lock();

    for grid in grids.iter_mut() {
        for tile in &mut grid.tiles.clone() {
            grid.close_tile_by_window_id(tile.window.id);
            tile.window.cleanup()?;
        }
    }

    Ok(())
}

pub fn with_current_grid<TFunction, TReturn>(f: TFunction) -> TReturn
where
    TFunction: Fn(&mut TileGrid) -> TReturn,
{
    with_grid_by_id(*WORKSPACE_ID.lock(), f)
}

pub fn with_grid_by_id<TFunction, TReturn>(id: i32, f: TFunction) -> TReturn
where
    TFunction: Fn(&mut TileGrid) -> TReturn,
{
    let mut grids = GRIDS.lock();
    let mut grid = grids.iter_mut().find(|g| g.id == id).unwrap();

    f(&mut grid)
}

fn on_quit() -> SystemResult {
    unmanage_everything()?;

    popup::cleanup();
    let remove_task_bar = {
        let config = CONFIG.lock();
        config.remove_task_bar
    };

    if remove_task_bar {
        task_bar::show_taskbars();
    }

    WIN_EVENT_LISTENER.stop();

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

    for display in DISPLAYS.lock().iter() {
        VISIBLE_WORKSPACES.lock().insert(display.id, 0);
    }

    info!("Starting hot reloading of config");
    config::hot_reloading::start();

    startup::set_launch_on_startup(CONFIG.lock().launch_on_startup)?;

    info!("Creating tray icon");
    tray::create()?;

    info!("Initializing workspaces");
    lazy_static::initialize(&WORKSPACES);

    toggle_work_mode::initialize()?;

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
                        tray::remove_icon(*tray::WINDOW.lock() as HWND);
                        on_quit()?;
                        break;
                    },
                    Event::ReloadConfig => {
                        info!("Reloading Config");

                        update_config(config::rhai::engine::parse_config().expect("Failed to load config"))
                    }
                }.map_err(|e| {
                    error!("{:?}", e);
                    crate::system::win::api::print_last_error();
                });
            }
        }
    }

    Ok(())
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");
    logging::setup().expect("Failed to setup logging");

    thread::spawn(|| loop {
        std::thread::sleep(Duration::from_secs(5));
        let deadlocks = deadlock::check_deadlock();
        if deadlocks.is_empty() {
            continue;
        }

        debug!("deadlock detected");
        debug!(
            "backtrace: \n{:?}",
            deadlocks.first().unwrap().first().unwrap().backtrace()
        );

        on_quit();
    });

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
