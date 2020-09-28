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
use lazy_static::lazy_static;
use log::debug;
use log::{error, info};
use parking_lot::{deadlock, Mutex};
use std::{collections::HashMap, process, sync::Arc};
use std::{thread, time::Duration};
use system::{DisplayId, SystemResult, WinEventListener};
use task_bar::Taskbar;
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
    pub grids: Vec<TileGrid>,
    pub visible_workspaces: HashMap<DisplayId, i32>,
    pub workspace_id: i32,
}

impl AppState {
    pub fn new() -> Self {
        let event_channel = EventChannel::default();
        let config = parse_config(&event_channel)
            .map_err(|e| error!("{}", e))
            .expect("Failed to load config");

        info!("Initializing displays");
        let displays = display::init(config.multi_monitor);

        let mut visible_workspaces = HashMap::new();

        for display in displays.iter() {
            visible_workspaces.insert(display.id, 0);
        }

        Self {
            work_mode: config.work_mode,
            displays,
            event_channel,
            additonal_rules: Vec::new(),
            window_event_listener: WinEventListener::default(),
            grids: (1..11)
                .map(|i| TileGrid::new(i, renderer::NativeRenderer::default()))
                .collect(),
            visible_workspaces,
            workspace_id: 1,
            config,
        }
    }

    /// TODO: maybe rename this function
    pub fn cleanup(&self) -> SystemResult {
        for grid in self.grids.iter_mut() {
            for tile in grid.tiles {
                grid.close_tile_by_window_id(tile.window.id);
                tile.window.cleanup()?;
            }
        }

        Ok(())
    }

    pub fn get_workspace_settings(&self, id: i32) -> Option<&WorkspaceSetting> {
        self.config.workspace_settings.iter().find(|s| s.id == id)
    }

    pub fn is_workspace_visible(&self, id: i32) -> bool {
        self.visible_workspaces.values().any(|v| *v == id)
    }

    pub fn change_workspace(&self, id: i32, force: bool) {
        let workspace_settings = self.config.workspace_settings;

        let grid = self.get_grid_by_id(id);

        if !force && grid.tiles.is_empty() {
            if let Some(settings) = self.get_workspace_settings(id) {
                if settings.monitor != -1 {
                    if let Some(display) = self.get_display_by_idx(settings.monitor) {
                        grid.display = display.clone();
                    } else {
                        error!("Invalid monitor {}. Workspace configuration for workspace {} is invalid!", settings.monitor, id);
                    }
                }
            }
        }

        debug!("Drawing the workspace");
        grid.draw_grid();
        debug!("Showing the workspace");
        grid.show();

        self.visible_workspaces.insert(grid.display.id, grid.id);
        self.workspace_id = id;
        self.redraw_app_bars();
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

    pub fn get_taskbars(&self) -> Vec<Taskbar> {
        self.displays
            .iter()
            .map(|d| d.taskbar)
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

    pub fn get_current_grid(&mut self) -> &mut TileGrid {
        self.get_grid_by_id(self.workspace_id)
    }

    pub fn get_grid_by_id(&mut self, id: i32) -> &mut TileGrid {
        self.grids.iter_mut().find(|g| g.id == id).unwrap()
    }
}

lazy_static! {
    pub static ref STATE: Mutex<AppState> = Mutex::new(AppState::new());
}

fn on_quit() -> SystemResult {
    let mut state = STATE.lock();

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
    use winapi::shared::windef::HWND;
    tray::remove_icon(*tray::WINDOW.lock() as HWND);
}

#[cfg(target_os = "windows")]
fn os_specific_setup() {
    info!("Creating tray icon");
    tray::create();
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let receiver = STATE.lock().event_channel.receiver.clone();
    let kb_manager = Arc::new(Mutex::new(KbManager::new(
        STATE.lock().config.keybindings.clone(),
    )));

    info!("Starting hot reloading of config");
    config::hot_reloading::start();

    let state = STATE.lock();

    startup::set_launch_on_startup(state.config.launch_on_startup);

    os_specific_setup();

    toggle_work_mode::initialize(&state, kb_manager.clone())?;

    info!("Listening for keybindings");
    kb_manager.clone().lock().start();

    loop {
        select! {
            recv(receiver) -> maybe_msg => {
                let msg = maybe_msg.unwrap();
                let _ = match msg {
                    Event::NewPopup(p) => Ok(p.create(&state.config)),
                    Event::Keybinding(kb) => event_handler::keybinding::handle(&mut *state, kb_manager.clone(), kb),
                    Event::RedrawAppBar => Ok(bar::redraw::redraw()),
                    Event::WinEvent(ev) => event_handler::winevent::handle(ev),
                    Event::Exit => {
                        on_quit()?;
                        break;
                    },
                    Event::ReloadConfig => {
                        info!("Reloading Config");
                        let new_config = parse_config(&state.event_channel)
                            .map_err(|e| error!("{}", e))
                            .expect("Failed to load config");
                        update_config(&state, kb_manager.clone(), new_config)
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
