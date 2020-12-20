#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate interpreter;

use bar::component::{Component, ComponentText};
use config::{rule::Rule, workspace_setting::WorkspaceSetting, Config};
use crossbeam_channel::select;
use direction::Direction;
use display::Display;
use event::Event;
use event::EventChannel;
use hot_reload::update_config;
use interpreter::{Dynamic, Function, Interpreter, Module, RuntimeError};
use itertools::Itertools;
use keybindings::{keybinding::Keybinding, KbManager};
use log::debug;
use log::{error, info};
use parking_lot::{deadlock, Mutex};
use popup::Popup;
use regex::Regex;
use split_direction::SplitDirection;
use std::fs::ReadDir;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::{process, sync::atomic::AtomicBool, sync::Arc};
use std::{thread, time::Duration};
use system::NativeWindow;
use system::{DisplayId, SystemResult, WinEventListener, WindowId};
use task_bar::Taskbar;
use tile_grid::TileGrid;
use win_event_handler::{win_event::WinEvent, win_event_type::WinEventType};
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
            keybindings_manager: KbManager::new(
                config.keybindings.clone(),
                config.mode_handlers.clone(),
            ),
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
        Self {
            work_mode: config.work_mode,
            displays: display::init(&config),
            keybindings_manager: KbManager::new(
                config.keybindings.clone(),
                config.mode_handlers.clone(),
            ),
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
        self.keybindings_manager = KbManager::new(
            self.config.keybindings.clone(),
            self.config.mode_handlers.clone(),
        );
    }

    /// TODO: maybe rename this function
    pub fn cleanup(&mut self) -> SystemResult {
        for d in self.displays.iter_mut() {
            for grid in d.grids.iter_mut() {
                grid.cleanup()?;
            }
        }

        Ok(())
    }

    pub fn move_workspace_to_monitor(&mut self, monitor: i32) -> SystemResult {
        let display = self.get_current_display_mut();

        if let Some(grid) = display
            .focused_grid_id
            .and_then(|id| display.remove_grid_by_id(id))
        {
            let config = self.config.clone();
            let new_display = self
                .get_display_by_idx_mut(monitor)
                .expect("Monitor with specified idx doesn't exist");

            let id = grid.id;

            new_display.grids.push(grid);
            new_display.focus_workspace(&config, id)?;
            self.workspace_id = id;
        }

        Ok(())
    }

    pub fn minimize_window(&mut self) -> SystemResult {
        let config = self.config.clone();
        let grid = self.get_current_grid_mut().unwrap();

        grid.modify_focused_window(|window| {
            window.minimize()?;
            window.cleanup()
        })?;

        grid.close_focused();

        let display = self.get_current_display_mut();
        display.refresh_grid(&config)?;

        Ok(())
    }

    pub fn close_window(&mut self) -> SystemResult {
        let config = self.config.clone();
        let grid = self.get_current_grid_mut().unwrap();

        grid.modify_focused_window(|window| {
            window.cleanup()?;
            window.close()
        })?;

        grid.close_focused();

        let display = self.get_current_display_mut();
        display.refresh_grid(&config)?;

        Ok(())
    }

    pub fn ignore_window(&mut self) -> SystemResult {
        if let Some(window) = self.get_current_grid().unwrap().get_focused_window() {
            let mut rule = Rule::default();

            let process_name = window.get_process_name();
            let pattern = format!("^{}$", process_name);

            debug!("Adding rule with pattern {}", pattern);

            rule.pattern = regex::Regex::new(&pattern).expect("Failed to build regex");
            rule.manage = false;

            self.additonal_rules.push(rule);

            self.toggle_floating();
        }

        Ok(())
    }

    pub fn move_window_to_workspace(&mut self, id: i32) -> SystemResult {
        let grid = self.get_current_grid_mut().unwrap();
        let window = grid.pop();

        window.map(|window| {
            self.get_grid_by_id_mut(id).unwrap().push(window);
            self.change_workspace(id, false);
        });

        Ok(())
    }

    pub fn toggle_fullscreen(&mut self) -> SystemResult {
        let config = self.config.clone();
        let display = self.get_current_display_mut();
        display.get_focused_grid_mut().unwrap().toggle_fullscreen();
        display.refresh_grid(&config)?;

        Ok(())
    }

    pub fn enter_work_mode(state_arc: Arc<Mutex<AppState>>) -> SystemResult {
        let mut this = state_arc.lock();
        if this.config.remove_task_bar {
            info!("Hiding taskbar");
            this.hide_taskbars();
        }

        if this.config.display_app_bar {
            drop(this);
            bar::create::create(state_arc.clone());
            this = state_arc.lock();
        }

        this.change_workspace(1, false);

        info!("Registering windows event handler");
        this.window_event_listener.start(&this.event_channel);

        Ok(())
    }

    pub fn leave_work_mode(state_arc: Arc<Mutex<AppState>>) -> SystemResult {
        let mut this = state_arc.lock();
        this.window_event_listener.stop();

        popup::cleanup()?;

        if this.config.display_app_bar {
            drop(this);
            bar::close_all(state_arc.clone());
            this = state_arc.lock();
        }

        if this.config.remove_task_bar {
            this.show_taskbars();
        }

        this.cleanup()?;
        Ok(())
    }

    pub fn toggle_work_mode(state_arc: Arc<Mutex<AppState>>) -> SystemResult {
        let mut this = state_arc.lock();
        this.work_mode = !this.work_mode;

        if !this.work_mode {
            drop(this);
            Self::leave_work_mode(state_arc)?;
        } else {
            drop(this);
            Self::enter_work_mode(state_arc)?;
        }

        Ok(())
    }

    pub fn swap(&mut self, direction: Direction) -> SystemResult {
        let config = self.config.clone();
        let display = self.get_current_display_mut();

        if let Some(grid) = display.get_focused_grid_mut() {
            grid.swap_focused(direction);
            display.refresh_grid(&config);
        }

        Ok(())
    }

    pub fn focus(&mut self, direction: Direction) -> SystemResult {
        let config = self.config.clone();
        let display = self.get_current_display_mut();

        if let Some(grid) = display.get_focused_grid_mut() {
            grid.focus(direction)?;
            display.refresh_grid(&config);
        }

        Ok(())
    }

    pub fn resize(&mut self, direction: Direction, amount: i32) -> SystemResult {
        let config = self.config.clone();
        let display = self.get_current_display_mut();

        if let Some(grid) = display.get_focused_grid_mut() {
            if !config.ignore_fullscreen_actions || !grid.is_fullscreened() {
                grid.trade_size_with_neighbor(grid.focused_id, direction, amount);
                info!("Resizing in the direction {:?} by {}", direction, amount);

                display.refresh_grid(&config)?;
            }
        }
        Ok(())
    }

    pub fn set_split_direction(&mut self, direction: SplitDirection) -> SystemResult {
        let display = self.get_current_display_mut();
        if let Some(grid) = display.get_focused_grid_mut() {
            grid.next_axis = direction;
        }
        Ok(())
    }

    pub fn toggle_floating(&mut self) -> SystemResult {
        let config = self.config.clone();

        let window =
            NativeWindow::get_foreground_window().expect("Failed to get foreground window");
        // The id of the grid that contains the window
        let maybe_grid_id = self
            .find_window(window.id)
            .and_then(|g| g.remove_by_window_id(window.id).map(|w| (g.id, w)))
            .map(|(id, mut w)| {
                debug!("Unmanaging window '{}' | {}", w.title, w.id);

                w.cleanup();

                id
            });

        if let Some(d) = maybe_grid_id.and_then(|id| self.find_grid(id)) {
            d.refresh_grid(&config);
        } else {
            self.event_channel
                .sender
                .clone()
                .send(Event::WinEvent(WinEvent {
                    typ: WinEventType::Show(true),
                    window,
                }))
                .expect("Failed to send WinEvent");
        }

        Ok(())
    }

    pub fn reset_column(&mut self) -> SystemResult {
        let config = self.config.clone();
        let display = self.get_current_display_mut();

        if let Some(g) = display.get_focused_grid_mut() {
            g.reset_column();
        }
        display.refresh_grid(&config)?;

        Ok(())
    }

    pub fn reset_row(&mut self) -> SystemResult {
        let config = self.config.clone();
        let display = self.get_current_display_mut();

        if let Some(g) = display.get_focused_grid_mut() {
            g.reset_row();
        }
        display.refresh_grid(&config)?;

        Ok(())
    }

    pub fn toggle_mode(&mut self, mode: String) {
        if self.keybindings_manager.get_mode() == Some(mode.clone()) {
            info!("Disabling {} mode", mode);
            self.keybindings_manager.leave_mode();
        } else {
            info!("Enabling {} mode", mode);
            self.keybindings_manager.enter_mode(&mode);
        }
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
        let current = self.get_current_display().id;
        if let Some(d) = self.find_grid(id) {
            let new = d.id;
            d.focus_workspace(&config, id);
            self.workspace_id = id;
            self.redraw_app_bars();
            if current != new {
                self.get_display_by_id(current)
                    .map(|d| d.refresh_grid(&config));
            }
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
    pub fn find_grid(&mut self, id: i32) -> Option<&mut Display> {
        for d in self.displays.iter_mut() {
            if let Some(_) = d.grids.iter().find(|g| g.id == id) {
                return Some(d);
            }
        }
        None
    }

    /// Returns the grid containing the window and its corresponding tile
    /// TODO: only return grid
    pub fn find_window(&mut self, id: WindowId) -> Option<&mut TileGrid> {
        for d in self.displays.iter_mut() {
            for g in d.grids.iter_mut() {
                if g.contains(id) {
                    return Some(g);
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

fn kb_from_args(callbacks_arc: Arc<Mutex<Vec<Function>>>, args: Vec<Dynamic>) -> Keybinding {
    let mut kb = Keybinding::from_str(&args[0].clone().as_str().unwrap()).unwrap();
    match &args[1] {
        Dynamic::Function {
            body,
            scope,
            arg_names,
            name,
        } => {
            let arg_names = arg_names.clone();
            let body = body.clone();
            let scope = scope.clone();

            let value = Function::new(&name.clone(), Some(scope.clone()), move |i, args| {
                i.call_fn(None, Some(scope.clone()), &arg_names, &args, &body)
            });

            let mut cbs = callbacks_arc.lock();
            let idx = cbs.len();
            cbs.push(value);
            kb.callback_id = idx;
        }
        Dynamic::RustFunction {
            name,
            callback,
            scope,
        } => {
            let callback = callback.clone();

            let value = Function::new(name, scope.clone(), move |i, args| {
                let args = args.clone();
                callback(i, args)
            });

            let mut cbs = callbacks_arc.lock();
            let idx = cbs.len();
            cbs.push(value);
            kb.callback_id = idx;
        }
        _ => todo!("{:?}", &args[1]),
    }
    kb
}

fn parse_config(
    state_arc: Arc<Mutex<AppState>>,
    callbacks_arc: Arc<Mutex<Vec<Function>>>,
    interpreter_arc: Arc<Mutex<Interpreter>>,
) -> Result<Config, String> {
    let config = Arc::new(Mutex::new(Config::default()));
    let is_init_inner = Arc::new(AtomicBool::new(true));
    let is_init_inner2 = is_init_inner.clone();
    let is_init = move || is_init_inner2.load(std::sync::atomic::Ordering::SeqCst);
    let mut interpreter = Interpreter::new();

    interpreter.debug = true;
    interpreter.source_locations = interpreter_arc.lock().source_locations.clone();

    let mut workspace = Module::new("workspace");

    let state = state_arc.clone();
    workspace = workspace.function("change", move |_, args| {
        let idx = number!(args[0])?;
        let mut state = state.lock();

        state.change_workspace(idx, true);

        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    workspace = workspace.function("move_to_monitor", move |_, args| {
        state.lock().move_workspace_to_monitor(number!(args[0])?);
        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    workspace = workspace.function("toggle_fullscreen", move |_, args| {
        state.lock().toggle_fullscreen();
        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    workspace = workspace.function("reset_row", move |_, args| {
        state.lock().reset_row();
        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    workspace = workspace.function("reset_col", move |_, args| {
        state.lock().reset_column();
        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    let cfg = config.clone();
    let is_init2 = is_init.clone();

    workspace = workspace.function("configure", move |_, args| {
        let id = *number!(&args[0])?;
        let config_ref = object!(&args[1])?;
        let config = config_ref.lock().unwrap();
        let mut settings = WorkspaceSetting::default();
        settings.id = id;

        for (key, val) in config.iter() {
            match key.as_str() {
                "text" => settings.text = string!(val)?.clone(),
                "monitor" => settings.monitor = *number!(val)?,
                _ => {}
            }
        }

        if is_init2() {
            cfg.lock().workspace_settings.push(settings);
        } else {
            state.lock().config.workspace_settings.push(settings);
        }

        Ok(Dynamic::Null)
    });


    let state = state_arc.clone();
    workspace = workspace.function("move_in", move |_, args| Ok(Dynamic::Null));

    let state = state_arc.clone();
    workspace = workspace.function("move_out", move |_, args| Ok(Dynamic::Null));

    let state = state_arc.clone();
    workspace = workspace.function("focus", move |_, args| {
        state
            .lock()
            .focus(Direction::from_str(string!(&args[0])?).unwrap());

        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    workspace = workspace.function("resize", move |_, args| {
        state.lock().resize(
            Direction::from_str(string!(&args[0])?).unwrap(),
            number!(args[1])?,
        );

        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    workspace = workspace.function("swap", move |_, args| {
        state
            .lock()
            .swap(Direction::from_str(string!(&args[0])?).unwrap());

        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    workspace = workspace.function("set_split_direction", move |_i, args| {
        state
            .lock()
            .set_split_direction(SplitDirection::from_str(string!(&args[0])?).unwrap());
        Ok(Dynamic::Null)
    });

    let mut window = Module::new("window");

    let state = state_arc.clone();
    window = window.function("get_title", move |_i, _args| {
        let state = state.lock();

        Ok(state
            .get_current_grid()
            .and_then(|g| g.get_focused_window())
            .and_then(|w| w.get_title().ok())
            .unwrap_or_default())
    });

    let state = state_arc.clone();
    window = window.function("minimize", move |_i, _args| {
        state.lock().minimize_window();
        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    window = window.function("toggle_floating", move |_i, _args| {
        state.lock().toggle_floating();
        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    window = window.function("ignore", move |_i, _args| {
        state.lock().ignore_window();
        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    window = window.function("close", move |_i, _args| {
        state.lock().close_window();
        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    window = window.function("move_to_workspace", move |_i, args| {
        state.lock().move_window_to_workspace(number!(args[0])?);
        Ok(Dynamic::Null)
    });

    let mut bar = Module::new("bar");
    let i_arc = interpreter_arc.clone();
    let state = state_arc.clone();
    let cfg = config.clone();
    let is_init2 = is_init.clone();

    bar = bar.function("configure", move |i, args| {
        let config_ref = object!(&args[0])?;
        let config = config_ref.lock().unwrap();

        for (key, val) in config.iter() {
            match key.as_str() {
                "height" => {
                    if is_init2() {
                        cfg.lock().bar.height = *number!(val)?;
                    } else {
                        state.lock().config.bar.height = *number!(val)?;
                    }
                }
                "font_size" => {
                    if is_init2() {
                        cfg.lock().bar.font_size = *number!(val)?;
                    } else {
                        state.lock().config.bar.font_size = *number!(val)?;
                    }
                }
                "font" => {
                    if is_init2() {
                        cfg.lock().bar.font = string!(val)?.clone();
                    } else {
                        state.lock().config.bar.font = string!(val)?.clone();
                    }
                }
                "color" => {
                    let mut color = *number!(val)?;
                    #[cfg(target_os = "windows")]
                    {
                        color = window::convert_color_to_winapi(color as u32) as i32;
                    }
                    if is_init2() {
                        cfg.lock().bar.color = color;
                    } else {
                        state.lock().config.bar.color = color;
                    }
                }
                "components" => {
                    let obj_ref = object!(val)?;
                    let obj = obj_ref.lock().unwrap();

                    for (key, val) in obj.iter() {
                        let raw_comps = val.clone().as_array().unwrap();
                        let mut comps = Vec::new();

                        for raw_comp in raw_comps {
                            let comp = Component::from_dynamic(i_arc.clone(), raw_comp)?;
                            comps.push(comp);
                        }

                        if is_init2() {
                            match key.as_ref() {
                                "left" => cfg.lock().bar.components.left = comps,
                                "center" => cfg.lock().bar.components.center = comps,
                                "right" => cfg.lock().bar.components.right = comps,
                                _ => {}
                            }
                        } else {
                            match key.as_ref() {
                                "left" => state.lock().config.bar.components.left = comps,
                                "center" => state.lock().config.bar.components.center = comps,
                                "right" => state.lock().config.bar.components.right = comps,
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Dynamic::Null)
    });

    let mut plugin = Module::new("plugin");
    let cfg = config.clone();

    plugin = plugin.function("install", move |i, args| {
        let name = string!(&args[0])?;
        let url = format!("https://www.github.com/{}", &name);
        let mut path = cfg.lock().plugins_path.clone();
        path.push(name.split("/").join("_"));

        if path.exists() {
            debug!("{} is already installed", name);
        } else {
            debug!("Installing {} from {}", name, url);
            Command::new("git")
                .arg("clone")
                .arg(&url)
                .arg(&path)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            path.push("plugin");

            i.source_locations.push(path.clone());
        }
        Ok(Dynamic::Null)
    });

    let cfg = config.clone();
    plugin = plugin.function("update", move |_i, _args| {
        for dir in get_plugins_path_iter() {
            if let Ok(dir) = dir {
                let name = dir.file_name().to_str().unwrap().to_string();

                let mut path = cfg.lock().plugins_path.clone();
                path.push(&name);

                let name = name.split("_").join("/");
                let url = format!("https://www.github.com/{}", name);

                let output = Command::new("git")
                    .arg("rev-parse")
                    .arg("--is-inside-work-tree")
                    .current_dir(&path)
                    .output()
                    .unwrap();

                let is_git_repo = output.stdout.iter().map(|&x| x as char).count() != 0;

                if !is_git_repo {
                    debug!("{} is not a git repo", name);
                    continue;
                }

                let output = Command::new("git")
                    .arg("rev-list")
                    .arg("HEAD...origin/master")
                    .arg("--count")
                    .current_dir(&path)
                    .output()
                    .unwrap();

                let has_updates =
                    output.stdout.iter().map(|&x| x as char).collect::<String>() != "0\n";

                if has_updates {
                    debug!("Updating {}", name);
                    Command::new("git")
                        .arg("pull")
                        .arg(&url)
                        .spawn()
                        .unwrap()
                        .wait()
                        .unwrap();
                } else {
                    debug!("{} is up to date", &name);
                }
            }
        }
        Ok(Dynamic::Null)
    });

    let cfg = config.clone();
    plugin = plugin.function("uninstall", move |_i, args| {
        let name = string!(&args[0])?;
        let mut path = cfg.lock().plugins_path.clone();
        path.push(name.split("/").join("_"));

        if path.exists() {
            debug!("Uninstalling {}", name);
            std::fs::remove_file(path).unwrap();
        } else {
            debug!("{} is not installed", name);
        }
        Ok(Dynamic::Null)
    });

    plugin = plugin.function("list", move |_, _| {
        let mut list: Vec<String> = Vec::new();

        for dir in get_plugins_path_iter() {
            if let Ok(dir) = dir {
                list.push(dir.path().to_str().unwrap().into());
            }
        }

        Ok(list)
    });

    let mut config_mod = Module::new("config");

    let state = state_arc.clone();
    let cfg = config.clone();
    let is_init2 = is_init.clone();
    config_mod = config_mod.function("increment", move |_i, args| {
        let (field, amount) = match args.len() {
            1 => (string!(&args[0])?, 1),
            _ => (string!(&args[0])?, *number!(&args[1])?),
        };

        if is_init2() {
            cfg.lock().set(string!(&args[0])?, string!(&args[1])?);
        } else {
            let mut cfg = cfg.lock().clone();
            cfg.increment_field(field, amount);
            update_config(state.clone(), cfg);
        }
        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    let cfg = config.clone();
    let is_init2 = is_init.clone();
    config_mod = config_mod.function("decrement", move |_i, args| {
        let (field, amount) = match args.len() {
            1 => (string!(&args[0])?, -1),
            _ => (string!(&args[0])?, *number!(&args[1])?),
        };

        if is_init2() {
            cfg.lock().set(string!(&args[0])?, string!(&args[1])?);
        } else {
            let mut cfg = cfg.lock().clone();
            cfg.decrement_field(field, amount);
            update_config(state.clone(), cfg);
        }

        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    let cfg = config.clone();
    let is_init2 = is_init.clone();
    config_mod = config_mod.function("toggle", move |_i, args| {
        if is_init2() {
            cfg.lock().set(string!(&args[0])?, string!(&args[1])?);
        } else {
            let mut cfg = cfg.lock().clone();
            cfg.toggle_field(string!(&args[0])?);
            update_config(state.clone(), cfg);
        }

        Ok(Dynamic::Null)
    });

    let cfg = config.clone();
    let state = state_arc.clone();
    let is_init2 = is_init.clone();
    config_mod = config_mod.function("set", move |_i, args| {
        if is_init2() {
            cfg.lock().set(string!(&args[0])?, string!(&args[1])?);
        } else {
            let mut cfg = cfg.lock().clone();
            cfg.set(string!(&args[0])?, string!(&args[1])?);
            update_config(state.clone(), cfg);
        }

        Ok(Dynamic::Null)
    });

    let cfg = config.clone();
    let state = state_arc.clone();
    let is_init2 = is_init.clone();
    config_mod = config_mod.function("enable", move |_i, args| {
        if is_init2() {
            cfg.lock().set(string!(&args[0])?, "true");
        } else {
            let mut cfg = cfg.lock().clone();
            cfg.set(string!(&args[0])?, "true");
            update_config(state.clone(), cfg);
        }

        Ok(Dynamic::Null)
    });

    let cfg = config.clone();
    let state = state_arc.clone();
    let is_init2 = is_init.clone();
    config_mod = config_mod.function("disable", move |_i, args| {
        if is_init2() {
            cfg.lock().set(string!(&args[0])?, "false");
        } else {
            let mut cfg = cfg.lock().clone();
            cfg.set(string!(&args[0])?, "false");
            update_config(state.clone(), cfg);
        }

        Ok(Dynamic::Null)
    });

    let mut rules = Module::new("rules");

    let cfg = config.clone();
    rules = rules.function("ignore", move |_, args| {
        let mut rule = Rule::default();
        rule.pattern = Regex::from_str(string!(&args[0])?).unwrap();
        rule.manage = false;

        cfg.lock().rules.push(rule);

        Ok(Dynamic::Null)
    });

    let cfg = config.clone();
    rules = rules.function("match", move |_, args| {
        let mut rule = Rule::default();
        rule.pattern = Regex::from_str(string!(&args[0])?).unwrap();

        let settings_ref = object!(&args[1])?;
        let settings = settings_ref.lock().unwrap();

        for (key, value) in settings.iter() {
            match key.as_str() {
                "has_custom_titlebar" => {
                    rule.has_custom_titlebar = *boolean!(value)?;
                }
                "chromium" => {
                    rule.chromium = *boolean!(value)?;
                }
                "firefox" => {
                    rule.firefox = *boolean!(value)?;
                }
                "manage" => {
                    rule.manage = *boolean!(value)?;
                }
                "workspace_id" => {
                    rule.workspace_id = *number!(value)?;
                }
                _ => todo!("{}", key),
            }
        }

        cfg.lock().rules.push(rule);

        Ok(Dynamic::Null)
    });

    let mut root = Module::new("nog")
        .variable("version", "<VERSION>")
        .variable("workspace", workspace)
        .variable("plugin", plugin)
        .variable("rules", rules)
        .variable("window", window)
        .variable("bar", bar)
        .variable("config", config_mod);

    let state = state_arc.clone();
    root = root.function("quit", move |_i, _args| {
        state.lock().event_channel.sender.send(Event::Exit);

        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    root = root.function("toggle_work_mode", move |_i, _args| {
        AppState::toggle_work_mode(state.clone());
        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    root = root.function("toggle_mode", move |_i, args| {
        state.lock().toggle_mode(string!(&args[0])?.clone());
        Ok(Dynamic::Null)
    });

    root = root.function("launch", move |_i, args| {
        system::api::launch_program(string!(&args[0])?.clone());
        Ok(Dynamic::Null)
    });

    let cbs = callbacks_arc.clone();
    let cfg = config.clone();
    let state = state_arc.clone();

    root = root.function("mode", move |i, args| {
        let cbs_arc = cbs.clone();
        let cfg = cfg.clone();

        let mode = string!(&args[0])?.clone();
        let mut cb = args[1].clone().as_fn().unwrap();
        let state2 = state.clone();

        let bind_fn = Function::new("bind", None, move |_, args| {
            // THIS FUNCTION
            let mut kb = kb_from_args(cbs_arc.clone(), args);
            kb.mode = Some(mode.clone());
            state2.lock().keybindings_manager.add_mode_keybinding(kb);
            Ok(Dynamic::Null)
        });

        cb.scope.set("bind".into(), bind_fn.into());

        let idx = cbs.lock().len();
        cbs.lock().push(cb);
        cfg.lock()
            .mode_handlers
            .insert(string!(&args[0])?.clone(), idx);

        Ok(())
    });

    let cfg = config.clone();
    let cbs = callbacks_arc.clone();
    root = root.function("bind", move |_i, args| {
        let kb = kb_from_args(cbs.clone(), args);
        cfg.lock().add_keybinding(kb);

        Ok(())
    });

    let cfg = config.clone();
    let cbs = callbacks_arc.clone();
    root = root.function("xbind", move |_i, args| {
        let mut kb = kb_from_args(cbs.clone(), args);
        kb.always_active = true;
        cfg.lock().add_keybinding(kb);

        Ok(())
    });

    interpreter.add_module(root);

    let mut config_path: PathBuf = dirs::config_dir().unwrap_or_default();
    config_path.push("nog");
    let mut plugins_path = config_path.clone();
    plugins_path.push("plugins");

    config.lock().path = config_path.clone();
    interpreter.source_locations.push(config_path.clone());

    if !config_path.exists() {
        debug!("nog folder doesn't exist yet. Creating the folder");
        std::fs::create_dir(config_path.clone()).map_err(|e| e.to_string())?;
    }

    if !plugins_path.exists() {
        debug!("plugins folder doesn't exist yet. Creating the folder");
        std::fs::create_dir(plugins_path.clone()).map_err(|e| e.to_string())?;
    }

    config.lock().plugins_path = plugins_path.clone();

    interpreter.source_locations.push(plugins_path.clone());

    config_path.push("config.ns");

    if !config_path.exists() {
        debug!("config file doesn't exist yet. Creating the file");
        if let Ok(mut file) = std::fs::File::create(config_path.clone()) {
            debug!("Initializing config with default values");
            // file.write_all(include_bytes!("../../../assets/default_config.nog"))
            //     .map_err(|e| e.to_string())?;
        }
    }

    debug!("Running config file");

    interpreter.execute_file(config_path)?;

    *interpreter_arc.lock() = interpreter;

    let cfg = config.lock();

    dbg!(&cfg.bar.color);

    is_init_inner.store(false, std::sync::atomic::Ordering::SeqCst);

    Ok(cfg.clone())
}

fn run(
    state_arc: Arc<Mutex<AppState>>,
    callbacks_arc: Arc<Mutex<Vec<Function>>>,
    interpreter_arc: Arc<Mutex<Interpreter>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let receiver = state_arc.lock().event_channel.receiver.clone();
    let sender = state_arc.lock().event_channel.sender.clone();

    info!("Starting hot reloading of config");
    config::hot_reloading::start(state_arc.clone());

    startup::set_launch_on_startup(state_arc.lock().config.launch_on_startup);

    os_specific_setup(state_arc.clone());

    if state_arc.lock().config.work_mode {
        AppState::enter_work_mode(state_arc.clone())?;
    }

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
                        p.create(state_arc.clone())?;
                        Ok(())
                    },
                    Event::ToggleAppbar(display_id) => {
                        let window = state_arc
                            .clone()
                            .lock()
                            .get_display_by_id(display_id)
                            .and_then(|d| d.appbar.as_ref())
                            .map(|bar| bar.window.get_native_window());

                        if let Some(win) = window {
                            if win.is_visible() {
                                println!("before");
                                win.hide();
                                println!("after");
                            } else {
                                win.show();
                            }
                        }
                        Ok(())
                    },
                    Event::Keybinding(kb) => {
                        debug!("Received keybinding {:?}", kb);
                        sender.send(Event::CallCallback { idx: kb.callback_id, is_mode_callback: false } ).unwrap();
                        Ok(())
                    },
                    Event::CallCallback { idx, is_mode_callback } => {
                        let cb = callbacks_arc.lock().get(idx).unwrap().clone();
                        if let Err(e) = cb.invoke(&mut interpreter_arc.lock(), vec![]) {
                            error!("{}", e.message());
                        }
                        if is_mode_callback {
                            state_arc.lock().keybindings_manager.sender.send(keybindings::ChanMessage::ModeCbExecuted);
                        }
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
                        match parse_config(state_arc.clone(), callbacks_arc.clone(), interpreter_arc.clone()) {
                            Ok(new_config) => update_config(state_arc.clone(), new_config),
                            Err(e) => {
                                sender.send(Event::NewPopup(Popup::new_error(vec![e])));
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

fn get_plugins_path_iter() -> ReadDir {
    let mut plugins_path: PathBuf = dirs::config_dir().unwrap_or_default();
    plugins_path.push("nog");
    plugins_path.push("plugins");

    plugins_path.read_dir().unwrap()
}

/// Fill source_locations of interpreter with plugin paths
fn load_plugin_source_locations(i: &mut Interpreter) {
    for dir in get_plugins_path_iter() {
        if let Ok(dir) = dir {
            let mut path = dir.path();
            path.push("plugin");
            i.source_locations.push(path);
        }
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    logging::setup().expect("Failed to setup logging");

    let state_arc = Arc::new(Mutex::new(AppState::default()));
    let callbacks_arc: Arc<Mutex<Vec<Function>>> = Arc::new(Mutex::new(Vec::new()));
    let mut interpreter = Interpreter::new();

    load_plugin_source_locations(&mut interpreter);

    let interpreter_arc = Arc::new(Mutex::new(interpreter));

    {
        let config = parse_config(
            state_arc.clone(),
            callbacks_arc.clone(),
            interpreter_arc.clone(),
        )
        .map_err(|e| {
            let state_arc = state_arc.clone();
            Popup::error(vec![e], state_arc);
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
    if let Err(e) = run(
        state_arc.clone(),
        callbacks_arc.clone(),
        interpreter_arc.clone(),
    ) {
        error!("An error occured {:?}", e);
        if let Err(e) = on_quit(&mut arc.lock()) {
            error!("Something happend when cleaning up. {}", e);
        }
    }
}
