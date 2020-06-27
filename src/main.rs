#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate strum_macros;

use crossbeam_channel::select;
use lazy_static::lazy_static;
use log::{debug, error, info};
use std::sync::Mutex;
use winapi::shared::windef::HWND;

mod app_bar;
mod config;
mod display;
mod event;
mod event_handler;
mod hot_key_manager;
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

use app_bar::RedrawAppBarReason;
use config::Config;
use display::Display;
use event::Event;
use event::EventChannel;
use tile_grid::TileGrid;
use workspace::Workspace;

lazy_static! {
    pub static ref WORK_MODE: Mutex<bool> = Mutex::new(CONFIG.work_mode);
    pub static ref CONFIG: Config = config::load().unwrap();
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

                    grid.height =
                        DISPLAY.lock().unwrap().height - CONFIG.margin * 2 - CONFIG.padding * 2;
                    grid.width =
                        DISPLAY.lock().unwrap().width - CONFIG.margin * 2 - CONFIG.padding * 2;

                    if CONFIG.display_app_bar {
                        grid.height -= CONFIG.app_bar_height;
                    }

                    grid
                })
                .collect::<Vec<TileGrid>>(),
        )
    };
    pub static ref WORKSPACES: Mutex<Vec<Workspace>> =
        { Mutex::new((1..11).map(Workspace::new).collect::<Vec<Workspace>>(),) };
    pub static ref WORKSPACE_ID: Mutex<i32> = Mutex::new(1);
}

#[cfg(debug_assertions)]
lazy_static! {
    pub static ref LOG_FILE: String = String::from("output.log");
}

#[cfg(not(debug_assertions))]
lazy_static! {
    pub static ref LOG_FILE: String = {
        let mut path = dirs::config_dir().unwrap();
        path.push("wwm");
        path.push("output.log");
        path.into_os_string().into_string().unwrap()
    };
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

    if CONFIG.remove_task_bar {
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

    startup::set_launch_on_startup(CONFIG.launch_on_startup)?;

    info!("Initializing display");
    lazy_static::initialize(&DISPLAY);

    info!("Initializing taskbar");
    task_bar::init();

    info!("Creating tray icon");
    tray::create()?;

    info!("Initializing workspaces");
    lazy_static::initialize(&WORKSPACES);

    if CONFIG.work_mode {
        if CONFIG.remove_task_bar {
            info!("Hiding taskbar");
            task_bar::hide();
        }

        if CONFIG.display_app_bar {
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
                        //lock Config
                        //parse Config
                        //if config is valid
                            //set config
                            //handle display_app_bar
                            //handle remove_task_bar
                            //handle remove_title_bar
                            //disable hot_key_manager with work_mode bindings
                            //enable hot_key_manager with work_mode bindings
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() {
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {:5} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(LOG_FILE.as_str()).unwrap())
        .apply()
        .unwrap();

    update::update().unwrap();

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
