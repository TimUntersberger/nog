#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate strum_macros;

use crate::event_handler::keybinding::toggle_work_mode::turn_work_mode_on;
use crate::event_handler::keybinding::toggle_work_mode::turn_work_mode_off;
use app_bar::RedrawAppBarReason;
use config::Config;
use crossbeam_channel::select;
use display::Display;
use event::Event;
use event::EventChannel;
use lazy_static::lazy_static;
use log::{debug, error, info};
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
        Mutex::new(config::load().expect("Failed to loading config"));
    pub static ref DISPLAY: Mutex<Display> = {
        let mut display = Display::default();
        display.init();
        Mutex::new(display)
    };
    pub static ref CHANNEL: EventChannel = EventChannel::default();
    pub static ref GRIDS: Mutex<Vec<TileGrid>> = {
        Mutex::new(
            (1..11)
                .map(|i| {
                    let mut grid = TileGrid::new(i);
                    let config = CONFIG.lock().unwrap();

                    grid.height =
                        DISPLAY.lock().unwrap().height - config.margin * 2 - config.padding * 2;
                    grid.width =
                        DISPLAY.lock().unwrap().width - config.margin * 2 - config.padding * 2;

                    if config.display_app_bar {
                        grid.height -= config.app_bar_height;
                    }

                    grid
                })
                .collect::<Vec<TileGrid>>(),
        )
    };
    pub static ref WORKSPACES: Mutex<Vec<Workspace>> =
        Mutex::new((1..11).map(Workspace::new).collect::<Vec<Workspace>>(),);
    pub static ref WORKSPACE_ID: Mutex<i32> = Mutex::new(1);
}

fn unmanage_everything() -> Result<(), util::WinApiResultError> {
    let mut grids = GRIDS.lock().unwrap();

    for grid in grids.iter_mut() {
        for tile in grid.tiles.clone() {
            grid.close_tile_by_window_id(tile.window.id);
            tile.window.reset_style()?;
            tile.window.reset_pos()?;
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

pub fn change_workspace(id: i32) -> Result<(), util::WinApiResultError> {
    let mut grids = GRIDS.lock().unwrap();
    let mut gid = WORKSPACE_ID.lock().unwrap();

    let old_id = *gid;
    *gid = id;

    let mut grid = grids.iter_mut().find(|g| g.id == *gid).unwrap();
    grid.visible = true;

    if old_id == id {
        debug!("Workspace is already selected");
        return Ok(());
    }

    debug!("Showing the next workspace");
    grid.visible = true;
    grid.draw_grid();
    grid.show();

    //without this delay there is a slight flickering of the background
    std::thread::sleep(std::time::Duration::from_millis(5));

    debug!("Hiding the current workspace");
    let mut grid = grids.iter_mut().find(|g| g.id == old_id).unwrap();
    grid.visible = false;
    grid.hide();

    drop(grids);
    drop(gid);

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

    info!("Starting hot reloading of config");
    config::hot_reloading::start();

    startup::set_launch_on_startup(CONFIG.lock().unwrap().launch_on_startup)?;

    info!("Initializing display");
    lazy_static::initialize(&DISPLAY);

    info!("Initializing taskbar");
    task_bar::init();

    info!("Creating tray icon");
    tray::create()?;

    info!("Initializing workspaces");
    lazy_static::initialize(&WORKSPACES);

    if CONFIG.lock().unwrap().work_mode {
        if CONFIG.lock().unwrap().remove_task_bar {
            info!("Hiding taskbar");
            task_bar::hide();
        }

        if CONFIG.lock().unwrap().display_app_bar {
            app_bar::create(&*DISPLAY.lock().unwrap())?;
        }

        info!("Registering windows event handler");
        win_event_handler::register()?;
    }

    change_workspace(1).expect("Failed to change workspace to ID@1");

    info!("Starting hot key manager");
    hot_key_manager::register()?;

    loop {
        select! {
            recv(receiver) -> maybe_msg => {
                let msg = maybe_msg.unwrap();
                //println!("{:?}", msg);
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
                        let mut old_config = CONFIG.lock().unwrap();
                        
                        turn_work_mode_off(true, old_config.display_app_bar, old_config.remove_task_bar)?;

                        let new_config = config::load().expect("Failed to load config");

                        turn_work_mode_on(new_config.display_app_bar, new_config.remove_task_bar)?;

                        *old_config = new_config;
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() {
    logging::setup().expect("Failed to setup logging");

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
}
