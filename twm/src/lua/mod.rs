use std::{str::FromStr, sync::Arc};

use regex::Regex;
use mlua::{Table, FromLua, Value, Error as LuaError};
use parking_lot::Mutex;

use log::{warn, error};

use crate::{
    direction::Direction,
    split_direction::SplitDirection,
    config::bar_config::BarConfig, config::Config, event::Event,
    keybindings::keybinding::Keybinding, keybindings::keybinding::KeybindingKind, AppState,
config::workspace_setting::WorkspaceSetting, config::rule::Rule};

mod runtime;
mod conversions;

pub use runtime::LuaRuntime;

static RUNTIME_FILE_CONTENT: &'static str = include_str!("../lua/runtime.lua");

#[derive(Clone)]
struct LuaBarConfigProxy {
    state: Arc<Mutex<AppState>>,
    config_copy: BarConfig,
}

impl mlua::UserData for LuaBarConfigProxy {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function(mlua::MetaMethod::Index, |_, (this, key): (Self, String)| {
            Ok(match key.as_str() {
                _ => (),
            })
        });

        methods.add_meta_function(
            mlua::MetaMethod::NewIndex,
            |lua, (this, key, val): (Self, String, Value)| {
                //TODO: Support components
                //
                //      pub components: BarComponentsConfig,

                //TODO: Also update the copy the same way
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
    config_copy: Config,
}

impl mlua::UserData for LuaConfigProxy {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function(mlua::MetaMethod::Index, |_, (this, key): (Self, String)| {
            Ok(match key.as_str() {
                "bar" => LuaBarConfigProxy {
                    state: this.state.clone(),
                    config_copy: this.config_copy.bar.clone(),
                },
                _ => todo!(),
            })
        });

        //TODO: Also update the copy the same way
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
    ($state_arc: expr, $lua: expr, $tbl: expr, $name: expr, $func_name: ident, $a1: ty, $a2: ty) => {
        let state = $state_arc.clone();
        def_fn!($lua, $tbl, $name, move |_, (a1, a2): ($a1, $a2)| {
            state.lock().$func_name(a1, a2).map_err(|e| LuaError::RuntimeError(e.to_string()))
        });
    };
    ($state_arc: expr, $lua: expr, $tbl: expr, $name: expr, $func_name: ident, $a1: ty) => {
        let state = $state_arc.clone();
        def_fn!($lua, $tbl, $name, move |_, a1: $a1| {
            state.lock().$func_name(a1).map_err(|e| LuaError::RuntimeError(e.to_string()))
        });
    };
    ($state_arc: expr, $lua: expr, $tbl: expr, $name: expr, $func_name: ident) => {
        let state = $state_arc.clone();
        def_fn!($lua, $tbl, $name, move |_, (): ()| {
            state.lock().$func_name().map_err(|e| LuaError::RuntimeError(e.to_string()))
        });
    };
}

fn setup_nog_global(state_arc: Arc<Mutex<AppState>>, rt: &LuaRuntime) {
    //TODO: bind functions need to set callback correctly
    rt.with_lua(move |lua| {
        let nog_tbl = lua.create_table()?;

        nog_tbl.set("version", option_env!("NOG_VERSION").unwrap_or("DEV"))?;

        let state = state_arc.clone();
        nog_tbl.set(
            "config",
            LuaConfigProxy {
                state: state.clone(),
                config_copy: state.lock().config.clone(),
            },
        )?;

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "quit", move |_, (): ()| {
            let _ = state.lock().event_channel.sender.send(Event::Exit);
            Ok(())
        });

        let state = state_arc.clone();
        def_fn!(lua, nog_tbl, "bind", move |_,
                                          (mode, key, cb): (
            String,
            String,
            Value
        )| {
            let mut kb = Keybinding::from_str(&key).unwrap();
            kb.kind = match mode.as_str() {
                "g" => KeybindingKind::Global,
                "w" => KeybindingKind::Work,
                _ => KeybindingKind::Normal,
            };
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
        l_def_ffi_fn!("move_to_ws", move_window_to_workspace, i32);

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
        l_def_ffi_fn!("move_to_monitor", move_workspace_to_monitor, i32);
        l_def_ffi_fn!("replace", move_workspace_to_workspace, i32);
        l_def_ffi_fn!("move_in", move_in, Direction);
        l_def_ffi_fn!("move_out", move_out, Direction);
        l_def_ffi_fn!("focus", focus, Direction);
        l_def_ffi_fn!("resize", resize, Direction, i32);
        l_def_ffi_fn!("swap", swap, Direction);
        l_def_ffi_fn!("set_split_direction", set_split_direction, SplitDirection);

        Ok(())
    })
}

pub fn setup_lua_rt(state_arc: Arc<Mutex<AppState>>) {
    let rt = state_arc.lock().lua_rt.clone();

    setup_nog_global(state_arc.clone(), &rt);

    rt.run_str("NOG_RUNTIME", RUNTIME_FILE_CONTENT);

    load_window_functions(state_arc.clone(), &rt).unwrap();
    load_workspace_functions(state_arc.clone(), &rt).unwrap();
    // load_plugin_functions(state_arc.clone(), &rt);
}
