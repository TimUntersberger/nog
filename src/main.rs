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
use keybindings::{
    key::Key, keybinding::Keybinding, keybinding_type::KeybindingType, modifier::Modifier,
    KbManager,
};
use log::debug;
use log::{error, info};
use parking_lot::{deadlock, Mutex};
use popup::Popup;
use std::{process, sync::Arc};
use std::{thread, time::Duration};
use system::{DisplayId, SystemResult, WinEventListener, WindowId};
use task_bar::Taskbar;
use tile::Tile;
use tile_grid::TileGrid;
use window::Window;

pub const NOG_BAR_NAME: &'static str = "nog_bar";
pub const NOG_POPUP_NAME: &'static str = "nog_popup";

#[macro_use]
#[allow(unused_macros)]
mod macros {
    /// logs the amount of time it took to execute the passed expression
    macro_rules! time {
        ($name: expr, $expr: expr) => {{
            let timer = std::time::Instant::now();
            let temp = $expr;
            log::debug!("{} took {:?}", $name, timer.elapsed());
            temp
        }};
    }
    /// sleeps for the given milliseconds
    macro_rules! sleep {
        ($ms: expr) => {
            std::thread::sleep(std::time::Duration::from_millis($ms))
        };
    }
    /// only runs the code if this is compiled on windows
    ///
    /// usage
    /// ```rust
    /// windows! {
    ///     // only runs on windows
    /// }
    /// ```
    /// TODO: correctly implement this
    macro_rules! windows {
        ($( $stmt:stmt )*) => {
            #[cfg(target_os = "windows")]
            {
                $(
                    $stmt
                )*
            };
        }
    }
    /// This macro either gets the Ok(..) value of the first expression or returns the second
    /// expression.
    macro_rules! fail_silent_with {
        ($expr: expr, $value: expr) => {
            match $expr {
                Ok(r) => r,
                Err(m) => return $value,
            };
        };
    }
    /// This macro either gets the Ok(..) value of the first expression or returns the second
    /// expression.
    /// This also prints the error using log::error
    macro_rules! fail_with {
        ($expr: expr, $value: expr) => {
            match $expr {
                Ok(r) => r,
                Err(m) => {
                    error!("{}", m);
                    return $value;
                }
            };
        };
    }
    /// This macro either gets the Ok(..) value of the passed expression or returns an Ok(()).
    /// This also prints the error using log::error
    macro_rules! fail {
        ($expr: expr) => {
            match $expr {
                Ok(r) => r,
                Err(m) => {
                    error!("{}", m);
                    return Ok(());
                }
            };
        };
    }
}

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

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Config,
    pub work_mode: bool,
    pub displays: Vec<Display>,
    pub event_channel: EventChannel,
    pub keybindings_manager: KbManager,
    pub additonal_rules: Vec<Rule>,
    pub window_event_listener: WinEventListener,
    pub workspace_id: i32,
}

impl Default for AppState {
    fn default() -> Self {
        let config = Config::default();
        Self {
            work_mode: true,
            displays: time!("initializing displays", display::init(&config)),
            keybindings_manager: KbManager::new(vec![Keybinding {
                typ: KeybindingType::CloseTile,
                mode: None,
                key: Key::Q,
                modifier: Modifier::ALT,
            }]),
            event_channel: EventChannel::default(),
            additonal_rules: Vec::new(),
            window_event_listener: WinEventListener::default(),
            workspace_id: 1,
            config,
        }
    }
}

impl AppState {
    pub fn new(config: Config) -> Self {
        // let config = parse_config(event_channel.sender.clone())?;
        Self {
            work_mode: config.work_mode,
            displays: display::init(&config),
            keybindings_manager: KbManager::new(config.keybindings.clone()),
            event_channel: EventChannel::default(),
            additonal_rules: Vec::new(),
            window_event_listener: WinEventListener::default(),
            workspace_id: 1,
            config,
        }
    }
    pub fn init(&mut self, config: Config) {
        self.config = config;
        self.work_mode = self.config.work_mode;
        self.displays = display::init(&self.config);
        self.keybindings_manager = KbManager::new(self.config.keybindings.clone());
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

    pub fn get_display_by_id(&self, id: DisplayId) -> Option<&Display> {
        self.displays.iter().find(|d| d.id == id)
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
            if let Some(grid) = d.grids.iter().find(|g| g.id == id).cloned() {
                return Some((d, grid));
            }
        }
        None
    }

    /// Returns the grid containing the window and its corresponding tile
    /// TODO: only return grid
    pub fn find_window(&mut self, id: WindowId) -> Option<(&mut TileGrid, Tile)> {
        for d in self.displays.iter_mut() {
            for g in d.grids.iter_mut() {
                if let Some(tile) = g.tiles.iter().find(|t| t.window.id == id).cloned() {
                    return Some((g, tile));
                }
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
        // have to hide the taskbars in a specific order for it to work (I know like wtf)

        // first hide primary display
        for d in &self.displays {
            if d.is_primary() {
                if let Some(tb) = &d.taskbar {
                    tb.window.hide();
                }
                break;
            }
        }

        // then the other ones
        for d in &self.displays {
            if !d.is_primary() {
                if let Some(tb) = &d.taskbar {
                    tb.window.hide();
                }
            }
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
    let sender = state_arc.lock().event_channel.sender.clone();

    info!("Starting hot reloading of config");
    config::hot_reloading::start(state_arc.clone());

    startup::set_launch_on_startup(state_arc.lock().config.launch_on_startup);

    os_specific_setup(state_arc.clone());

    toggle_work_mode::initialize(state_arc.clone())?;

    info!("Listening for keybindings");
    state_arc
        .lock()
        .keybindings_manager
        .start(state_arc.clone());

    loop {
        select! {
            recv(receiver) -> maybe_msg => {
                let msg = maybe_msg.unwrap();
                let _ = match msg {
                    Event::NewPopup(mut p) => {
                        p.create(state_arc.clone());
                        Ok(())
                    },
                    Event::ToggleAppbar(display_id) => {
                        if let Some(bar) = state_arc
                            .clone()
                            .lock()
                            .get_display_by_id(display_id)
                            .and_then(|d| d.appbar.clone()) {
                            let win = bar.window.get_native_window();

                            if win.is_visible() {
                                println!("before");
                                //TODO: This causes a deadlock
                                //      probably because i use ShowWindow which is sync
                                win.hide();
                                println!("after");
                            } else {
                                win.show();
                            }
                        }
                        Ok(())
                    },
                    Event::Keybinding(kb) => {
                        event_handler::keybinding::handle(state_arc.clone(), kb);
                        Ok(())
                    },
                    Event::RedrawAppBar => {
                        let windows = state_arc.lock().displays.iter().map(|d| d.appbar.as_ref()).flatten().map(|b| b.window.clone()).collect::<Vec<Window>>();

                        for window in windows {
                            window.redraw();
                        }

                        Ok(())
                    },
                    Event::WinEvent(ev) => event_handler::winevent::handle(&mut state_arc.lock(), ev),
                    Event::Exit => {
                        on_quit(&mut state_arc.lock())?;
                        break;
                    },
                    Event::ReloadConfig => {
                        info!("Reloading Config");
                        match parse_config(state_arc.clone()) {
                            Ok(new_config) => update_config(state_arc.clone(), new_config),
                            Err(e) => {
                                sender.send(Event::NewPopup(Popup::new()
                                    .with_padding(5)
                                    .with_text(&[&e])
                                ));
                                Ok(())
                            }

                        }
                    },
                    Event::UpdateBarSections(display_id, left, center, right) => {
                        let mut state = state_arc.lock();
                        for d in state.displays.iter_mut() {
                            if d.id == display_id {
                                if let Some(bar) = d.appbar.as_mut() {
                                    bar.left = left;
                                    bar.center = center;
                                    bar.right = right;
                                    break;
                                }
                            }
                        }
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
    std::env::set_var("RUST_BACKTRACE", "1");
    logging::setup().expect("Failed to setup logging");

    let state_arc = Arc::new(Mutex::new(AppState::default()));

    {
        let config = parse_config(state_arc.clone())
            .map_err(|e| {
                Popup::new()
                    .with_padding(5)
                    .with_text(&[&e, "", "(Press Alt+Q to close)"])
                    .create(state_arc.clone())
                    .unwrap();
            })
            .unwrap_or_default();
        state_arc.lock().init(config)
    }

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

        on_quit(&mut arc.lock()).unwrap();
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
