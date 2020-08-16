use super::engine::MODE;
use crate::{
    bar::component::Component,
    config::{
        bar_config::BarConfig, update_channel::UpdateChannel, Config, Rule, WorkspaceSetting,
    },
    keybindings::{keybinding::Keybinding, keybinding_type::KeybindingType},
};
use log::error;
use regex::Regex;
use rhai::{Array, Dynamic, Engine, Map, ParseError};
use std::{cell::RefCell, rc::Rc, str::FromStr, time::Duration, sync::{Mutex, Arc}};

#[macro_use]
mod macros;

fn set_config(config: &mut Config, key: String, value: Dynamic) {
    set!(bool, config, use_border, key, value);
    set!(i32, config, min_width, key, value);
    set!(i32, config, min_height, key, value);
    set!(bool, config, work_mode, key, value);
    set!(bool, config, light_theme, key, value);
    set!(bool, config, multi_monitor, key, value);
    set!(bool, config, launch_on_startup, key, value);
    set!(i32, config, outer_gap, key, value);
    set!(i32, config, inner_gap, key, value);
    set!(bool, config, remove_title_bar, key, value);
    set!(bool, config, remove_task_bar, key, value);
    set!(bool, config, display_app_bar, key, value);
    if key == "update_interval" {
        if value.type_name().to_string() != "i32" {
            error!(
                "{} has to be of type {} not {}",
                "update_interval",
                "i32",
                value.type_name()
            );
        } else {
            config.update_interval = Duration::from_secs(value.clone().cast::<u64>() * 60);
        }
    }
    if key == "default_update_channel" {
        if value.type_name().to_string() != "string" {
            error!(
                "{} has to be of type {} not {}",
                "default_update_channel",
                "String",
                value.type_name()
            );
        } else {
            config.default_update_channel = Some(value.clone().as_str().unwrap().to_string());
        }
    }
}

pub fn init(engine: &mut Engine, config: &mut Arc<Mutex<Config>>) -> Result<(), Box<ParseError>> {
    let cfg = config.clone();
    engine.register_custom_syntax(
        &["bind", "$expr$", "$expr$"], // the custom syntax
        0, // the number of new variables declared within this custom syntax
        move |engine, ctx, scope, inputs| {
            let key = get_string!(engine, ctx, scope, inputs, 0);
            let binding = get_type!(engine, ctx, scope, inputs, 1, KeybindingType);
            let mut kb = Keybinding::from_str(&key).unwrap();

            kb.typ = binding;
            kb.mode = MODE.lock().unwrap().clone();

            cfg.lock().unwrap().keybindings.push(kb);

            Ok(().into())
        },
    )?;

    let cfg = config.clone();
    engine.register_custom_syntax(
        &["bind_range", "$expr$", "$expr$", "$expr$", "$ident$"], // the custom syntax
        0, // the number of new variables declared within this custom syntax
        move |engine, ctx, scope, inputs| {
            let from = get_int!(engine, ctx, scope, inputs, 0);
            let to = get_int!(engine, ctx, scope, inputs, 1);
            let modifier = get_string!(engine, ctx, scope, inputs, 2);
            let binding_name = get_variable_name!(inputs, 3);

            for i in from..to + 1 {
                if i == 11 {
                    break;
                }

                let key = if i == 10 {
                    format!("{}+{}", modifier, 0)
                } else {
                    format!("{}+{}", modifier, i)
                };

                let binding: KeybindingType =
                    engine.eval_expression(&format!("{}({})", binding_name, i))?;

                let mut kb = Keybinding::from_str(&key).unwrap();

                kb.typ = binding;
                kb.mode = MODE.lock().unwrap().clone();

                cfg.lock().unwrap().keybindings.push(kb);
            }

            Ok(().into())
        },
    )?;

    let cfg = config.clone();
    engine.register_custom_syntax(
        &["bar", "$expr$"], // the custom syntax
        0,                  // the number of new variables declared within this custom syntax
        move |engine, ctx, scope, inputs| {
            let settings = get_map!(engine, ctx, scope, inputs, 0);
            let mut bar_config: BarConfig = BarConfig::default();

            for (key, val) in settings {
                if key.to_string() == "components" {
                    bar_config.components.empty();
                    let map = val.cast::<Map>();

                    for (key, val) in map {
                        let key = key.to_string();
                        let components = val.cast::<Array>();

                        for v in components {
                            let component = v.cast::<Component>();

                            let list = match key.as_str() {
                                "left" => &mut bar_config.components.left,
                                "center" => &mut bar_config.components.center,
                                "right" => &mut bar_config.components.right,
                                _ => panic!(),
                            };

                            list.push(component);
                        }
                    }
                } else {
                    set!(i32, bar_config, color, key, val);
                    set!(i32, bar_config, height, key, val);
                    set!(String, bar_config, font, key, val);
                    set!(i32, bar_config, font_size, key, val);
                }
            }

            cfg.lock().unwrap().bar = bar_config;

            Ok(().into())
        },
    )?;

    let cfg = config.clone();
    engine.register_custom_syntax(
        &["set", "$ident$", "$expr$"], // the custom syntax
        0, // the number of new variables declared within this custom syntax
        move |engine, ctx, scope, inputs| {
            let key = get_variable_name!(inputs, 0);
            let value = get_dynamic!(engine, ctx, scope, inputs, 1);
            let mut config = cfg.lock().unwrap();

            set_config(&mut config, key, value);

            Ok(().into())
        },
    )?;

    let cfg = config.clone();
    engine.register_custom_syntax(
        &["enable", "$ident$"], // the custom syntax
        0,                      // the number of new variables declared within this custom syntax
        move |_engine, _ctx, _scope, inputs| {
            let key = get_variable_name!(inputs, 0);
            let mut config = cfg.lock().unwrap();

            set_config(&mut config, key, true.into());

            Ok(().into())
        },
    )?;

    let cfg = config.clone();
    engine.register_custom_syntax(
        &["disable", "$ident$"], // the custom syntax
        0,                       // the number of new variables declared within this custom syntax
        move |_engine, _ctx, _scope, inputs| {
            let key = get_variable_name!(inputs, 0);
            let mut config = cfg.lock().unwrap();

            set_config(&mut config, key, false.into());

            Ok(().into())
        },
    )?;

    let cfg = config.clone();
    engine.register_custom_syntax(
        &["rule", "$expr$", "$expr$"], // the custom syntax
        0, // the number of new variables declared within this custom syntax
        move |engine, ctx, scope, inputs| {
            let pattern = get_string!(engine, ctx, scope, inputs, 0);
            let settings = get_map!(engine, ctx, scope, inputs, 1);
            let mut rule = Rule::default();

            for (key, value) in settings.iter().map(|(k, v)| (k.to_string(), v)) {
                set!(bool, rule, manage, key, value);
                set!(bool, rule, has_custom_titlebar, key, value);
                set!(bool, rule, firefox, key, value);
                set!(bool, rule, chromium, key, value);
                set!(i32, rule, workspace_id, key, value);
            }

            rule.pattern = Regex::new(&format!("^{}$", pattern)).map_err(|e| e.to_string())?;

            cfg.lock().unwrap().rules.push(rule);

            Ok(().into())
        },
    )?;

    let cfg = config.clone();
    engine.register_custom_syntax(
        &["update_channel", "$expr$", "$expr$"], // the custom syntax
        0, // the number of new variables declared within this custom syntax
        move |engine, ctx, scope, inputs| {
            let name = get_string!(engine, ctx, scope, inputs, 0);
            let settings = get_map!(engine, ctx, scope, inputs, 1);
            let mut update_channel = UpdateChannel::default();

            update_channel.name = name;

            for (key, value) in settings.iter().map(|(k, v)| (k.to_string(), v)) {
                set!(String, update_channel, branch, key, value);
                set!(String, update_channel, repo, key, value);
                set!(String, update_channel, version, key, value);
            }

            cfg.lock().unwrap().update_channels.push(update_channel);

            Ok(().into())
        },
    )?;

    let cfg = config.clone();
    engine.register_custom_syntax(
        &["ignore", "$expr$"], // the custom syntax
        0,                     // the number of new variables declared within this custom syntax
        move |engine, ctx, scope, inputs| {
            let pattern = get_string!(engine, ctx, scope, inputs, 0);
            let mut rule = Rule::default();

            rule.pattern = Regex::new(&format!("^{}$", pattern)).map_err(|e| e.to_string())?;
            rule.manage = false;

            cfg.lock().unwrap().rules.push(rule);

            Ok(().into())
        },
    )?;

    let cfg = config.clone();
    engine.register_custom_syntax(
        &["workspace", "$expr$", "$expr$"], // the custom syntax
        0, // the number of new variables declared within this custom syntax
        move |engine, ctx, scope, inputs| {
            let id = get_int!(engine, ctx, scope, inputs, 0);
            let settings = get_map!(engine, ctx, scope, inputs, 1);
            let mut workspace = WorkspaceSetting::default();

            workspace.id = id;

            for (key, value) in settings.iter().map(|(k, v)| (k.to_string(), v)) {
                set!(i32, workspace, monitor, key, value);
                set!(String, workspace, text, key, value);
            }

            cfg.lock().unwrap().workspace_settings.push(workspace);

            Ok(().into())
        },
    )?;

    let cfg = config.clone();
    engine.register_custom_syntax(
        &["mode", "$expr$", "$expr$", "$block$"], // the custom syntax
        0, // the number of new variables declared within this custom syntax
        move |engine, ctx, scope, inputs| {
            let name = get_string!(engine, ctx, scope, inputs, 0);
            let key = get_string!(engine, ctx, scope, inputs, 1);

            let mut kb = Keybinding::from_str(&key).unwrap();
            kb.typ = KeybindingType::ToggleMode(name.clone());

            cfg.lock().unwrap().keybindings.push(kb);

            *MODE.lock().unwrap() = Some(name);

            engine.eval_expression_tree(ctx, scope, inputs.get(2).unwrap())?;

            *MODE.lock().unwrap() = None;

            Ok(().into())
        },
    )?;

    Ok(())
}
