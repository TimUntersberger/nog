use std::{str::FromStr, sync::Arc};

use mlua::{Table, Variadic, Value, Error as LuaError};
use parking_lot::Mutex;

use crate::{
    direction::Direction,
    split_direction::SplitDirection,
    config::bar_config::BarConfig, config::Config, event::Event,
    keybindings::keybinding::Keybinding, keybindings::keybinding::KeybindingKind, AppState,
};

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
                use mlua::FromLua;
                //TODO: Support components
                //
                //      pub components: BarComponentsConfig,

                //TODO: Also update the copy the same way
                match key.as_str() {
                    "height" => {
                        this.state.lock().config.bar.height = FromLua::from_lua(val, lua).unwrap()
                    }
                    "color" => {
                        this.state.lock().config.bar.color = FromLua::from_lua(val, lua).unwrap()
                    }
                    "font" => {
                        this.state.lock().config.bar.font = FromLua::from_lua(val, lua).unwrap()
                    }
                    "font_size" => {
                        this.state.lock().config.bar.font_size =
                            FromLua::from_lua(val, lua).unwrap()
                    }
                    _ => todo!(),
                };

                Ok(())
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
        use mlua::Value;

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
                use mlua::FromLua;

                match key.as_str() {
                    "work_mode" => {
                        this.state.lock().config.work_mode = FromLua::from_lua(val, lua).unwrap()
                    }
                    "display_app_bar" => {
                        this.state.lock().config.display_app_bar =
                            FromLua::from_lua(val, lua).unwrap()
                    }
                    "remove_task_bar" => {
                        this.state.lock().config.remove_task_bar =
                            FromLua::from_lua(val, lua).unwrap()
                    }
                    "remove_title_bar" => {
                        this.state.lock().config.remove_title_bar =
                            FromLua::from_lua(val, lua).unwrap()
                    }
                    "launch_on_startup" => {
                        this.state.lock().config.launch_on_startup =
                            FromLua::from_lua(val, lua).unwrap()
                    }
                    "multi_monitor" => {
                        this.state.lock().config.multi_monitor =
                            FromLua::from_lua(val, lua).unwrap()
                    }
                    "use_border" => {
                        this.state.lock().config.use_border = FromLua::from_lua(val, lua).unwrap()
                    }
                    "min_width" => {
                        this.state.lock().config.min_width = FromLua::from_lua(val, lua).unwrap()
                    }
                    "min_height" => {
                        this.state.lock().config.min_height = FromLua::from_lua(val, lua).unwrap()
                    }
                    "light_theme" => {
                        this.state.lock().config.light_theme = FromLua::from_lua(val, lua).unwrap()
                    }
                    "outer_gap" => {
                        this.state.lock().config.outer_gap = FromLua::from_lua(val, lua).unwrap()
                    }
                    "inner_gap" => {
                        this.state.lock().config.inner_gap = FromLua::from_lua(val, lua).unwrap()
                    }
                    "ignore_fullscreen_actions" => {
                        this.state.lock().config.ignore_fullscreen_actions =
                            FromLua::from_lua(val, lua).unwrap()
                    }
                    _ => todo!(),
                };

                Ok(())
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
