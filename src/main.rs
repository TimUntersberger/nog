//#![windows_subsystem = "windows"]

#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate strum_macros;

use crossbeam_channel::select;
use lazy_static::lazy_static;
use log::{debug, error, info};
use std::sync::Mutex;

mod app_bar;
mod config;
mod tray;
mod display;
mod event;
mod event_handler;
mod hot_key_manager;
mod task_bar;
mod tile;
mod tile_grid;
mod util;
mod win_event_handler;
mod window;
mod workspace;

use config::Config;
use display::Display;
use app_bar::RedrawAppBarReason;
use event::Event;
use event::EventChannel;
use tile_grid::TileGrid;
use workspace::Workspace;

lazy_static! {
    pub static ref CONFIG: Config = config::load().unwrap();
    pub static ref DISPLAY: Mutex<Display> = {
        let mut display = Display::default();
        display.init();
        return Mutex::new(display);
    };
    pub static ref CHANNEL: EventChannel = EventChannel::default();
    pub static ref GRIDS: Mutex<Vec<TileGrid>> = {
        return Mutex::new(
            (1..11)
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
            (1..11)
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

    CHANNEL.sender.clone().send(Event::RedrawAppBar(RedrawAppBarReason::Workspace));

    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let receiver = CHANNEL.receiver.clone();
    let sender = CHANNEL.sender.clone();

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
    }

    info!("Creating tray icon");
    tray::create()?;

    info!("Initializing workspaces");
    lazy_static::initialize(&WORKSPACES);

    info!("Registering windows event handler");
    win_event_handler::register()?;

    info!("Starting hot key manager");
    hot_key_manager::register()?;

    loop {
        select! {
            recv(receiver) -> msg => {
                match msg.unwrap() {
                    Event::Keybinding(kb) => event_handler::keybinding::handle(kb)?,
                    Event::RedrawAppBar(reason) => app_bar::redraw(reason),
                    Event::WinEvent(ev) => event_handler::winevent::handle(ev)?,
                    Event::Exit => {
                        on_quit()?;
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() {
    env_logger::init();
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
}
