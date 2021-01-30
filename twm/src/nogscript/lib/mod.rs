use crate::update_config;
use crate::{
    bar::component,
    bar::component::{Component, ComponentText},
    config::{workspace_setting::WorkspaceSetting, Config},
    direction::Direction,
    keybindings::keybinding::Keybinding,
    split_direction::SplitDirection,
    system, window, AppState, Event, Rule,
};
use crate::{get_plugins_path_iter, popup::Popup};
use interpreter::{Dynamic, Function, Interpreter, Module, RuntimeError};
use itertools::Itertools;
use log::debug;
use parking_lot::Mutex;
use regex::Regex;
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

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

    if let Some(always_active) = args.get(2) {
        if always_active.is_true() {
            kb.always_active = true;
        }
    }

    kb
}

pub fn create_root_module(
    is_init: impl Fn() -> bool + Clone + Send + Sync + 'static,
    state_arc: Arc<Mutex<AppState>>,
    callbacks_arc: Arc<Mutex<Vec<Function>>>,
    interpreter_arc: Arc<Mutex<Interpreter>>,
    config: Arc<Mutex<Config>>,
) -> Module {
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
    workspace = workspace.function("move_in", move |_, args| {
        state
            .lock()
            .move_in(Direction::from_str(string!(&args[0])?).unwrap());

        Ok(Dynamic::Null)
    });

    let state = state_arc.clone();
    workspace = workspace.function("move_out", move |_, args| {
        state
            .lock()
            .move_out(Direction::from_str(string!(&args[0])?).unwrap());

        Ok(Dynamic::Null)
    });

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

    bar = bar.variable("components", {
        let state = state_arc.clone();
        let mut m = Module::new("components").function("workspaces", move |_, _| {
            Ok(component::workspaces::create(state.clone()).into_dynamic(state.clone()))
        });

        let state = state_arc.clone();
        m = m.function("time", move |_, args| {
            let text = string!(&args[0])?.clone();
            Ok(component::time::create(text).into_dynamic(state.clone()))
        });

        let state = state_arc.clone();
        m = m.function("date", move |_, args| {
            let text = string!(&args[0])?.clone();
            Ok(component::date::create(text).into_dynamic(state.clone()))
        });

        let state = state_arc.clone();
        m = m.function("padding", move |_, args| {
            let count = *number!(&args[0])?;
            Ok(component::padding::create(count).into_dynamic(state.clone()))
        });

        let state = state_arc.clone();
        m = m.function("current_window", move |_, _| {
            Ok(component::current_window::create(state.clone()).into_dynamic(state.clone()))
        });

        let state = state_arc.clone();
        m = m.function("active_mode", move |_, _| {
            Ok(component::active_mode::create(state.clone()).into_dynamic(state.clone()))
        });

        let state = state_arc.clone();
        m = m.function("split_direction", move |_, args| {
            let vertical = string!(&args[0])?.clone();
            let horizontal = string!(&args[1])?.clone();
            Ok(
                component::split_direction::create(state.clone(), vertical, horizontal)
                    .into_dynamic(state.clone()),
            )
        });

        let state = state_arc.clone();
        m = m.function("text", move |_, args| {
            let text = string!(&args[0])?.clone();
            Ok(Component::new("Text", move |_| {
                let text = text.clone();
                Ok(vec![ComponentText::new().with_display_text(text)])
            })
            .into_dynamic(state.clone()))
        });
        m
    });

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
                    let mut state = state
                        .try_lock_for(Duration::from_millis(100))
                        .ok_or("Failed to get state lock")?;

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
                                "left" => state.config.bar.components.left = comps,
                                "center" => state.config.bar.components.center = comps,
                                "right" => state.config.bar.components.right = comps,
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
        if let Ok(dirs) = get_plugins_path_iter() {
            for dir in dirs {
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

        if let Ok(dirs) = get_plugins_path_iter() {
            for dir in dirs {
                if let Ok(dir) = dir {
                    list.push(dir.path().to_str().unwrap().into());
                }
            }
        }

        Ok(list)
    });

    let mut popup = Module::new("popup");
    let state = state_arc.clone();
    popup = popup.function("create", move |_i, args| {
        let mut popup = Popup::new();
        match args.len() {
            0 => {}
            _ => {
                let map_ref = object!(&args[0])?;
                let map = map_ref.lock().unwrap();

                for (key, value) in map.iter() {
                    match key.as_str() {
                        "text" => match value {
                            Dynamic::String(x) => {
                                popup = popup.with_text(vec![x]);
                            }
                            Dynamic::Array(items) => {
                                let items = items.lock().unwrap();
                                let mut content = Vec::new();

                                for item in items.iter() {
                                    content.push(string!(item)?);
                                }

                                popup = popup.with_text(content);
                            }
                            x => {
                                return Err(RuntimeError::UnexpectedType {
                                    expected: "String | Array".into(),
                                    actual: x.type_name(),
                                });
                            }
                        },
                        "padding" => {
                            popup = popup.with_padding(*number!(value)?);
                        }
                        _ => {}
                    }
                }
            }
        };

        popup
            .create(state.clone())
            .map_err(|err| format!("{:?}", err))?;

        Ok(Dynamic::Null)
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
            let mut cfg = state.lock().config.clone();
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
            let mut cfg = state.lock().config.clone();
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
            let mut cfg = state.lock().config.clone();
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
            let mut cfg = state.lock().config.clone();
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
            let mut cfg = state.lock().config.clone();
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
            let mut cfg = state.lock().config.clone();
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
        .variable("version", option_env!("NOG_VERSION").unwrap_or("DEV"))
        .variable("workspace", workspace)
        .variable("plugin", plugin)
        .variable("rules", rules)
        .variable("window", window)
        .variable("popup", popup)
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
    root = root.function("bind_arr", move |_i, args| {
        let modifier = string!(&args[0])?;
        let callback = args[1].clone().as_fn()?;
        let arr_ref = array!(&args[2])?;
        let always_active = if let Some(val) = args.get(3) {
            *boolean!(val)?
        } else {
            false
        };

        let arr = arr_ref.lock().unwrap();

        for (i, value) in arr.iter().enumerate() {
            let value = value.clone();
            let callback = callback.clone();
            let args = vec![
                format!("{}+{}", modifier, i).into(),
                Dynamic::RustFunction {
                    name: "bind_arr_gen_fn".into(),
                    callback: Arc::new(move |i, _| callback.invoke(i, vec![value.clone()])),
                    scope: None,
                },
                always_active.into(),
            ];

            let kb = kb_from_args(cbs.clone(), args);
            cfg.lock().add_keybinding(kb);
        }

        Ok(())
    });

    let cfg = config.clone();
    let cbs = callbacks_arc.clone();
    root = root.function("bind_map", move |_i, args| {
        let modifier = string!(&args[0])?;
        let callback = args[1].clone().as_fn()?;
        let map_ref = object!(&args[2])?;
        let always_active = if let Some(val) = args.get(3) {
            *boolean!(val)?
        } else {
            false
        };

        let map = map_ref.lock().unwrap();

        for (key, value) in map.iter() {
            let value = value.clone();
            let callback = callback.clone();
            let args = vec![
                format!("{}+{}", modifier, key).into(),
                Dynamic::RustFunction {
                    name: "bind_map_gen_fn".into(),
                    callback: Arc::new(move |i, _| callback.invoke(i, vec![value.clone()])),
                    scope: None,
                },
                always_active.into(),
            ];

            let kb = kb_from_args(cbs.clone(), args);
            cfg.lock().add_keybinding(kb);
        }

        Ok(())
    });

    root
}
