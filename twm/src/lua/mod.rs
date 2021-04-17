use std::{str::FromStr, sync::Arc};

use mlua::{Error as LuaError, FromLua, Function, Lua, Table, ToLua, Value};
use parking_lot::Mutex;
use regex::Regex;

use log::{error, warn};

use crate::{
    bar::component::Component, config::rule::Rule, config::workspace_setting::WorkspaceSetting,
    direction::Direction, event::Event, keybindings::keybinding::Keybinding,
    keybindings::keybinding::KeybindingKind, split_direction::SplitDirection, system::DisplayId,
    system::WindowId, AppState,
system};

mod conversions;
mod runtime;

pub use runtime::{LuaRuntime, get_err_msg};

/// This is macro is necessary, because if you use the default way of type checking input
/// (specifying the type in the function declaration) the error is different than how it would look
/// like when you check the type at runtime.
///
/// This macro basically does the same thing, but throws a
/// runtime error on the lua side instead of a CallbackError on the rust side.
macro_rules! validate {
    ($lua: tt, $x: ident : $type: tt) => {
        match <$type>::from_lua($x.clone(), $lua) {
            Ok(x) => Ok(x),
            Err(_) => {
                let msg = format!("Expected `{}` to be of type `{}` (found `{}`)", stringify!($x), stringify!($type), $x.type_name());
                Err(LuaError::RuntimeError(msg))
            }
        }
    };
    ($lua: ident, { $($x: ident : $type: ty),* }) => {
        $(let $x = validate!($lua, $x: $type)?;)*
    }
}

macro_rules! validate_tbl_prop {
    ($lua: tt, $tbl: tt, $x: ident, $t: tt) => {
        let $x = match $tbl.get::<_, Value>(stringify!($x)) {
            Err(_) => {
                return Err(LuaError::RuntimeError(format!(
                    "A component needs to have a {} field",
                    stringify!($x)
                )))
            }
            Ok(x) => x,
        };
        let $x = validate!($lua, $x: $t)?;
    };
}

#[derive(Clone)]
struct LuaBarConfigProxy {
    state: Arc<Mutex<AppState>>,
}

impl LuaBarConfigProxy {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self { state }
    }
}

impl mlua::UserData for LuaBarConfigProxy {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function(mlua::MetaMethod::Index, |_, (this, key): (Self, String)| {
            Ok(match key.as_str() {
				"color" => Value::Integer(this.state.lock().config.bar.color as i64),
                _ => Value::Nil,
            })
        });

        methods.add_meta_function(
            mlua::MetaMethod::NewIndex,
            |lua, (this, key, val): (Self, String, Value)| {
                if key == "components" {
                    let tbl = validate!(lua, val: Table)?;

                    if let Ok(tbl) = tbl.get::<_, Table>("left") {
                        for res in tbl.sequence_values::<Table>() {
                            if let Ok(comp_tbl) = res {
                                validate_tbl_prop!(lua, comp_tbl, name, String);
                                validate_tbl_prop!(lua, comp_tbl, render, Function);
                                let id = LuaRuntime::add_callback(lua, render)?;
                                let state_arc = this.state.clone();
                                let comp = Component::new(&name, move |disp_id| {
                                    let rt = state_arc.lock().lua_rt.clone();
                                    let res = rt.with_lua(|lua| {
                                        let cb = LuaRuntime::get_callback(&lua, id)?;
                                        cb.call(disp_id.0)
                                    });

									Ok(match res {
										Err(e) => {
											error!("{}", crate::lua::get_err_msg(&e));
											vec![]
										},
										Ok(x) => x
									})
                                });
                                this.state.lock().config.bar.components.left.push(comp);
                            }
                        }
                    }

                    return Ok(());
                }
                //TODO: Support components
                //
                //      pub components: BarComponentsConfig,
                macro_rules! map_props {
                    ($($prop_name: ident),*) => {
                        match key.as_str() {
                            $(stringify!($prop_name) => {
                                this.state.lock().config.bar.$prop_name = FromLua::from_lua(val, lua).unwrap()
                            })*,
                            x => {
                                warn!("Unknown bar config property '{}'", x);
                            }
                        }
                    }
                }
                Ok(map_props!(height, color, font, font_size))
            },
        );
    }
}

#[derive(Clone)]
struct LuaConfigProxy {
    state: Arc<Mutex<AppState>>,
}

impl LuaConfigProxy {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        Self { state }
    }
}

impl mlua::UserData for LuaConfigProxy {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function(mlua::MetaMethod::Index, |lua, (this, key): (Self, String)| {
            Ok(match key.as_str() {
                "bar" => Value::UserData(lua.create_userdata(LuaBarConfigProxy::new(this.state.clone()))?),
                "light_theme" => Value::Boolean(this.state.lock().config.light_theme),
                _ => Value::Nil,
            })
        });

        methods.add_meta_function(
            mlua::MetaMethod::NewIndex,
            |lua, (this, key, val): (Self, String, Value)| {
                if key == "rules" {
                    match val {
                        Value::Table(tbl) => {
                            for pair in tbl.pairs::<String, Table>() {
                                if let Ok((key, val)) = pair {
                                    let mut rule = Rule::default();
                                    //TODO: remove unwrap
                                    rule.pattern = Regex::new(&key).unwrap();

                                    for pair in val.pairs::<String, Value>() {
                                        if let Ok((key, val)) = pair {
                                            match key.as_str() {
                                                "ignore" => rule.manage = !FromLua::from_lua(val, lua)?,
                                                "chromium" => rule.chromium = FromLua::from_lua(val, lua)?,
                                                "firefox" => rule.firefox = FromLua::from_lua(val, lua)?,
                                                "has_custom_titlebar" => rule.has_custom_titlebar = FromLua::from_lua(val, lua)?,
                                                "workspace_id" => rule.workspace_id = FromLua::from_lua(val, lua)?,
                                                _ => {}
                                            }
                                        }
                                    }

                                    this.state.lock().config.rules.push(rule);
                                }
                            }
                        },
                        x => {
                            error!("nog.config.rules has to be a table (found {})", x.type_name());
                        }
                    }
                    return Ok(());
                } 

                if key == "workspaces" {
                    match val {
                        Value::Table(tbl) => {
                            for pair in tbl.pairs::<i32, Table>() {
                                if let Ok((key, val)) = pair {
                                    let mut settings = WorkspaceSetting::default();
                                    settings.id = key;

                                    for pair in val.pairs::<String, Value>() {
                                        if let Ok((key, val)) = pair {
                                            match key.as_str() {
                                                "text" => settings.text = FromLua::from_lua(val, lua)?,
                                                "monitor" => settings.monitor = FromLua::from_lua(val, lua)?,
                                                _ => {}
                                            }
                                        }
                                    }

                                    this.state.lock().config.workspace_settings.push(settings);
                                }
                            }
                        },
                        x => {
                            error!("nog.config.workspaces has to be a table (found {})", x.type_name());
                        }
                    }
                    return Ok(());
                }

                macro_rules! map_props {
                    ($($prop_name: ident),*) => {
                        match key.as_str() {
                            $(stringify!($prop_name) => {
                                this.state.lock().config.$prop_name = FromLua::from_lua(val, lua).unwrap()
                            })*,
                            x => {
                                warn!("Unknown config property '{}'", x);
                            }
                        }
                    }
                }

                Ok(map_props!(
                    work_mode,
                    display_app_bar,
                    remove_task_bar,
                    remove_title_bar,
                    launch_on_startup,
                    multi_monitor,
                    use_border,
                    min_height,
                    min_width,
                    light_theme,
                    outer_gap,
                    inner_gap,
                    ignore_fullscreen_actions
                ))
            },
        );
    }
}

macro_rules! def_fn {
    ($lua: expr, $tbl: expr, $name: expr, $func: expr) => {
        $tbl.set($name, $lua.create_function($func)?)?;
    };
}

/// This is used to map a lua function to a rust function.
macro_rules! def_ffi_fn {
    ($state_arc: expr, $lua: expr, $tbl: expr, $name: expr, $func_name: ident, $a1:ident : $a1t: ty, $a2:ident : $a2t: ty) => {
        let state = $state_arc.clone();
        def_fn!($lua, $tbl, $name, move |lua, ($a1, $a2): (Value, Value)| {
            let $a1 = validate!(lua, $a1: $a1t)?;
            let $a2 = validate!(lua, $a2: $a2t)?;
            state
                .lock()
                .$func_name($a1, $a2)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });
    };
    ($state_arc: expr, $lua: expr, $tbl: expr, $name: expr, $func_name: ident, $a1: ident : $a1t: tt) => {
        let state = $state_arc.clone();
        def_fn!($lua, $tbl, $name, move |lua, $a1: Value| {
            let $a1 = validate!(lua, $a1: $a1t)?;
            state
                .lock()
                .$func_name($a1)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });
    };
    ($state_arc: expr, $lua: expr, $tbl: expr, $name: expr, $func_name: ident) => {
        let state = $state_arc.clone();
        def_fn!($lua, $tbl, $name, move |_, (): ()| {
            state
                .lock()
                .$func_name()
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });
    };
}

fn setup_nog_global(state_arc: Arc<Mutex<AppState>>, rt: &LuaRuntime) {
    //TODO: bind functions need to set callback correctly
    rt.with_lua(move |lua| {
        let nog_tbl = lua.create_table()?;
        let cb_tbl = lua.create_table()?;

        nog_tbl.set("__callbacks", cb_tbl)?;
        nog_tbl.set("version", option_env!("NOG_VERSION").unwrap_or("DEV"))?;

        let state = state_arc.clone();
        nog_tbl.set("config", LuaConfigProxy::new(state.clone()))?;

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "quit", move |_, (): ()| {
            let _ = state.lock().event_channel.sender.send(Event::Exit);
            Ok(())
        });

        def_fn!(lua, nog_tbl, "launch", move |lua, name: Value| {
            validate!(lua, { name: String });
            system::api::launch_program(name)?;
            Ok(())
        });

        let state = state_arc.clone();
        def_fn!(
            lua,
            nog_tbl,
            "get_active_ws_of_display",
            move |lua, display_id: Value| {
                validate!(lua, { display_id: i32 });
                let state_g = state.lock();
                let grids = state_g
                    .get_display_by_id(DisplayId(display_id))
                    .unwrap()
                    .get_active_grids();

                Ok(grids.iter().map(|g| g.id).collect::<Vec<_>>())
            }
        );

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "get_kb_mode", move |_, (): ()| {
            Ok(state.lock().keybindings_manager.try_get_mode().unwrap())
        });

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "toggle_work_mode", move |_, (): ()| {
            AppState::toggle_work_mode(state.clone())
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "is_ws_focused", move |lua, ws_id: Value| {
            validate!(lua, { ws_id: i32 });
            Ok(state.lock().workspace_id == ws_id)
        });

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "get_win_title", move |lua, win_id: Value| {
            validate!(lua, { win_id: i32 });
            let win_id = WindowId(win_id);
            Ok(state
                .lock()
                .find_grid_containing_window(win_id)
                .and_then(|g| g.get_window(win_id))
                .map(|w| w.title.clone()))
        });

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "get_ws_info", move |lua, ws_id: Value| {
            validate!(lua, { ws_id: i32 });
            Ok(state.lock().get_grid_by_id(ws_id).map(|ws| {
                let tbl = lua.create_table().unwrap();
                tbl.set("id", ws.id);
                tbl.set("is_fullscreen", ws.is_fullscreened());
                tbl.set("is_empty", ws.is_empty());
                tbl.set("split_direction", ws.next_axis.to_string());
                let windows = ws.get_windows().iter().map(|w| w.id.0).collect::<Vec<_>>();
                tbl.set("windows", windows);
                tbl
            }))
        });

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "get_current_display_id", move |_, (): ()| {
            Ok(state.lock().get_current_display().id.0)
        });

        def_fn!(lua, nog_tbl, "scale_color", move |lua, (color, factor): (Value, Value)| {
			validate!(lua, {
				color: i32,
				factor: f64
			});

            Ok(crate::util::scale_color(color, factor))
        });

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "get_current_win", move |_, (): ()| {
            let win_id = state
                .lock()
                .get_current_display()
                .get_focused_grid()
                .and_then(|g| g.get_focused_window())
                .map(|w| w.id.0);

            Ok(win_id)
        });

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "get_current_ws", move |_, (): ()| {
            let ws_id = state
                .lock()
                .get_current_display()
                .get_focused_grid()
                .map(|ws| ws.id);

            Ok(ws_id)
        });

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "get_ws_text", move |lua, ws_id: Value| {
            validate!(lua, { ws_id: i32 });
            Ok(state.lock().get_ws_text(ws_id))
        });

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "bind", move |lua,
                                            (mode, key, cb): (
            Value,
            Value,
            Value
        )| {
            validate!(lua, {
                mode: String,
                key: String,
                cb: Function
            });
            let mut kb = Keybinding::from_str(&key).unwrap();
            kb.kind = match mode.as_str() {
                "g" => KeybindingKind::Global,
                "w" => KeybindingKind::Work,
                _ => KeybindingKind::Normal,
            };

            let id = LuaRuntime::add_callback(lua, cb)?;

            kb.callback_id = id as usize;
            state.lock().add_keybinding(kb);
            Ok(())
        });

        let globals = lua.globals();
        globals.set("nog", nog_tbl)?;

        Ok(())
    })
    .unwrap();
}

fn load_window_functions(state_arc: Arc<Mutex<AppState>>, rt: &LuaRuntime) -> mlua::Result<()> {
    rt.with_lua(|lua| {
        let nog_tbl = lua.globals().get::<_, Table>("nog")?;

        /// A local version of the `def_ffi_fn` macro for ease of use.
        ///
        /// **Note**: Also prefixes the name with `win_`
        macro_rules! l_def_ffi_fn {
            ($name: expr, $($rest: tt)*) => {
                def_ffi_fn!(state_arc, lua, nog_tbl, format!("{}_{}", "win", $name), $($rest)*);
            };
        }

        l_def_ffi_fn!("get_title", get_window_title);
        l_def_ffi_fn!("minimize", minimize_window);
        l_def_ffi_fn!("toggle_floating", toggle_floating);
        l_def_ffi_fn!("ignore", ignore_window);
        l_def_ffi_fn!("close", close_window);
        l_def_ffi_fn!("move_to_ws", move_window_to_workspace, ws_id: i32);

        Ok(())
    })
}

fn load_workspace_functions(state_arc: Arc<Mutex<AppState>>, rt: &LuaRuntime) -> mlua::Result<()> {
    rt.with_lua(|lua| {
        let nog_tbl = lua.globals().get::<_, Table>("nog")?;

        /// A local version of the `def_ffi_fn` macro for ease of use.
        ///
        /// **Note**: Also prefixes the name with `ws_`
        macro_rules! l_def_ffi_fn {
            ($name: expr, $($rest: tt)*) => {
                def_ffi_fn!(state_arc, lua, nog_tbl, format!("{}_{}", "ws", $name), $($rest)*);
            };
        }

        l_def_ffi_fn!("toggle_fullscreen", toggle_fullscreen);
        l_def_ffi_fn!("reset_row", reset_row);
        l_def_ffi_fn!("reset_col", reset_column);
        l_def_ffi_fn!(
            "move_to_monitor",
            move_workspace_to_monitor,
            monitor_id: i32
        );
        l_def_ffi_fn!("replace", move_workspace_to_workspace, ws_id: i32);
        l_def_ffi_fn!("change", emit_change_workspace, ws_id: i32);
        l_def_ffi_fn!("move_in", move_in, direction: Direction);
        l_def_ffi_fn!("move_out", move_out, direction: Direction);
        l_def_ffi_fn!("focus", focus, direction: Direction);
        l_def_ffi_fn!("resize", resize, direction: Direction, amount: i32);
        l_def_ffi_fn!("swap", swap, direction: Direction);
        l_def_ffi_fn!(
            "set_split_direction",
            set_split_direction,
            direction: SplitDirection
        );

        Ok(())
    })
}

pub fn setup_lua_rt(state_arc: Arc<Mutex<AppState>>) {
    let rt = state_arc.lock().lua_rt.clone();

    setup_nog_global(state_arc.clone(), &rt);

    rt.run_str("NOG_INSPECT", include_str!("../lua/inspect.lua"));
    rt.run_str("NOG_RUNTIME", include_str!("../lua/runtime.lua"));

    load_window_functions(state_arc.clone(), &rt).unwrap();
    load_workspace_functions(state_arc.clone(), &rt).unwrap();
    // load_plugin_functions(state_arc.clone(), &rt);
    //  plug_update
    //  plug_install
    //  plug_uninstall
    //  plug_update_all
}
