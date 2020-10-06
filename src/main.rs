#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate strum_macros;

use config::{rhai::engine::parse_config, rule::Rule, workspace_setting::WorkspaceSetting, Config};
use crossbeam_channel::select;
use display::Display;
use event::Event;
use event::EventChannel;
use event_handler::keybinding::toggle_work_mode;
use hot_reload::update_config;
use keybindings::KbManager;
use log::debug;
use log::{error, info};
use parking_lot::{deadlock, lock_api::MutexGuard, Mutex};
use std::{process, sync::Arc};
use std::{thread, time::Duration};
use system::{SystemResult, WinEventListener, WindowId};
use task_bar::Taskbar;
use tile::Tile;
use tile_grid::TileGrid;

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
mod renderer;
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

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    pub work_mode: bool,
    pub displays: Vec<Display>,
    pub event_channel: EventChannel,
    pub additonal_rules: Vec<Rule>,
    pub window_event_listener: WinEventListener,
    pub workspace_id: i32,
}

impl AppState {
    pub fn new() -> Self {
        let event_channel = EventChannel::default();
        let config = parse_config(&event_channel)
            .map_err(|e| error!("{}", e))
            .expect("Failed to load config");

        info!("Initializing displays");
        let displays = display::init(&config);

        Self {
            work_mode: config.work_mode,
            displays,
            event_channel,
            additonal_rules: Vec::new(),
            window_event_listener: WinEventListener::default(),
            workspace_id: 1,
            config,
        }
    }

    /// TODO: maybe rename this function
    pub fn cleanup(&mut self) -> SystemResult {
        for d in self.displays.iter_mut() {
            for grid in d.grids.iter_mut() {
                for tile in grid.tiles.iter_mut() {
                    tile.window.cleanup()?;
                }
            }
        }

        Ok(())
    }

    pub fn get_workspace_settings(&self, id: i32) -> Option<&WorkspaceSetting> {
        self.config.workspace_settings.iter().find(|s| s.id == id)
    }

    pub fn is_workspace_visible(&self, id: i32) -> bool {
        self.displays
            .iter()
            .find(|d| d.focused_grid_id == Some(id))
            .is_some()
    }

    pub fn change_workspace(&mut self, id: i32, _force: bool) {
        let config = self.config.clone();
        if let Some((d, _)) = self.find_grid(id) {
            d.focus_workspace(&config, id);
            self.workspace_id = id;
            self.redraw_app_bars();
        }
    }

    pub fn redraw_app_bars(&self) {
        debug!("Sending redraw-app-bar event");
        self.event_channel
            .sender
            .send(Event::RedrawAppBar)
            .expect("Failed to send redraw-app-bar event");
    }

    pub fn get_display_by_idx(&self, idx: i32) -> Option<&Display> {
        let x: usize = if idx == -1 {
            0
        } else {
            std::cmp::max(self.displays.len() - (idx as usize), 0)
        };

        self.displays.get(x)
    }

    pub fn get_display_by_idx_mut(&mut self, idx: i32) -> Option<&mut Display> {
        let x: usize = if idx == -1 {
            0
        } else {
            std::cmp::max(self.displays.len() - (idx as usize), 0)
        };

        self.displays.get_mut(x)
    }

    pub fn get_taskbars(&self) -> Vec<&Taskbar> {
        self.displays
            .iter()
            .map(|d| d.taskbar.as_ref())
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect()
    }

    /// Returns the display containing the grid and the grid
    /// TODO: only return display
    pub fn find_grid(&mut self, id: i32) -> Option<(&mut Display, TileGrid)> {
        for d in self.displays.iter_mut() {
            let grid = d.grids.iter().find(|g| g.id == id).unwrap().clone();
            return Some((d, grid));
        }
        None
    }

    /// Returns the grid containing the window and its corresponding tile
    /// TODO: only return grid
    pub fn find_window(&mut self, id: WindowId) -> Option<(&mut TileGrid, Tile)> {
        for d in self.displays.iter_mut() {
            for g in d.grids.iter_mut() {
                let tile = g.tiles.iter().find(|t| t.window.id == id).unwrap().clone();
                return Some((g, tile));
            }
        }

        None
    }

    pub fn get_taskbars_mut(&mut self) -> Vec<&mut Taskbar> {
        self.displays
            .iter_mut()
            .map(|d| d.taskbar.as_mut())
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect()
    }

    pub fn show_taskbars(&self) {
        for tb in self.get_taskbars() {
            tb.window.show();
        }
    }

    pub fn hide_taskbars(&self) {
        for tb in self.get_taskbars() {
            tb.window.hide();
        }
    }

    pub fn get_current_display_mut(&mut self) -> &mut Display {
        let workspace_id = self.workspace_id;
        self.displays
            .iter_mut()
            .find(|d| d.grids.iter().any(|g| g.id == workspace_id))
            .unwrap()
    }

    pub fn get_current_display(&self) -> &Display {
        self.displays
            .iter()
            .find(|d| d.grids.iter().any(|g| g.id == self.workspace_id))
            .unwrap()
    }

    pub fn get_current_grid_mut(&mut self) -> Option<&mut TileGrid> {
        self.get_grid_by_id_mut(self.workspace_id)
    }

    pub fn get_current_grid(&self) -> Option<&TileGrid> {
        self.get_grid_by_id(self.workspace_id)
    }

    pub fn get_grids_mut(&mut self) -> Vec<&mut TileGrid> {
        self.displays
            .iter_mut()
            .map(|d| d.grids.iter_mut())
            .flatten()
            .collect()
    }

    pub fn get_grids(&self) -> Vec<&TileGrid> {
        self.displays
            .iter()
            .map(|d| d.grids.iter())
            .flatten()
            .collect()
    }

    pub fn get_grid_by_id_mut(&mut self, id: i32) -> Option<&mut TileGrid> {
        self.get_grids_mut().into_iter().find(|g| g.id == id)
    }

    pub fn get_grid_by_id(&self, id: i32) -> Option<&TileGrid> {
        self.get_grids().into_iter().find(|g| g.id == id)
    }
}

fn on_quit(state: &mut AppState) -> SystemResult {
    os_specific_cleanup();

    state.cleanup()?;

    popup::cleanup();

    if state.config.remove_task_bar {
        state.show_taskbars();
    }

    state.window_event_listener.stop();

    process::exit(0);
}

#[cfg(target_os = "windows")]
fn os_specific_cleanup() {
    if let Some(window) = tray::WINDOW.lock().as_ref() {
        tray::remove_icon(window.id.into());
    }
}

#[cfg(target_os = "windows")]
fn os_specific_setup(state: Arc<Mutex<AppState>>) {
    info!("Creating tray icon");
    tray::create(state);
}

fn run(state_arc: Arc<Mutex<AppState>>) -> Result<(), Box<dyn std::error::Error>> {
    let receiver = state_arc.lock().event_channel.receiver.clone();
    let kb_manager = Arc::new(Mutex::new(KbManager::new(
        state_arc.lock().config.keybindings.clone(),
    )));

    info!("Starting hot reloading of config");
    config::hot_reloading::start(state_arc.clone());

    startup::set_launch_on_startup(state_arc.lock().config.launch_on_startup);

    os_specific_setup(state_arc.clone());

    toggle_work_mode::initialize(state_arc.clone(), kb_manager.clone())?;

    info!("Listening for keybindings");
    kb_manager.clone().lock().start(state_arc.clone());

    loop {
        select! {
            recv(receiver) -> maybe_msg => {
                let msg = maybe_msg.unwrap();
                let _ = match msg {
                    Event::NewPopup(mut p) => Ok(p.create(state_arc.clone())),
                    Event::Keybinding(kb) => {
                        event_handler::keybinding::handle(state_arc.clone(), kb_manager.clone(), kb)
                    },
                    Event::RedrawAppBar => Ok(()), // TODO: redraw appbars
                    Event::WinEvent(ev) => event_handler::winevent::handle(&mut state_arc.lock(), ev),
                    Event::Exit => {
                        on_quit(&mut state_arc.lock())?;
                        break;
                    },
                    Event::ReloadConfig => {
                        info!("Reloading Config");
                        let new_config = parse_config(&state_arc.lock().event_channel)
                            .map_err(|e| error!("{}", e))
                            .expect("Failed to load config");
                        update_config(state_arc.clone(), kb_manager.clone(), new_config)
                    },
                    Event::UpdateBarSections(display_id, left, center, right) => {
                        // TODO: implement
                        Ok(())
                    },
                    Event::ChangeWorkspace(id, force) => {
                        state_arc.lock().change_workspace(id, force);
                        Ok(())
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

    let state_arc = Arc::new(Mutex::new(AppState::new()));
    let arc = state_arc.clone();

    thread::spawn(move || loop {
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

        on_quit(&mut arc.lock());
    });

    info!("");

    let arc = state_arc.clone();
    ctrlc::set_handler(move || {
        if let Err(e) = on_quit(&mut arc.lock()) {
            error!("Something happend when cleaning up. {}", e);
        }
    })
    .unwrap();

    let arc = state_arc.clone();
    if let Err(e) = run(state_arc.clone()) {
        error!("An error occured {:?}", e);
        if let Err(e) = on_quit(&mut arc.lock()) {
            error!("Something happend when cleaning up. {}", e);
        }
    }
}
