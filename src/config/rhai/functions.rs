use crate::{
    config::{WorkspaceSetting, Rule},
    direction::Direction,
    keybindings::{keybinding::Keybinding, keybinding_type::KeybindingType},
    split_direction::SplitDirection,
};
use rhai::{Array, Dynamic, Engine, RegisterFn};
use std::str::FromStr;

pub fn init(engine: &mut Engine) {
    engine.register_fn("push", |list: &mut Array, item: Keybinding| {
        list.push(Dynamic::from(Box::new(item)))
    });
    engine.register_fn("push", |list: &mut Array, item: Rule| {
        list.push(Dynamic::from(Box::new(item)))
    });
    engine.register_fn("push", |list: &mut Array, item: WorkspaceSetting| {
        list.push(Dynamic::from(Box::new(item)))
    });

    engine.register_fn("close_tile", || KeybindingType::CloseTile);
    engine.register_fn("minimize_tile", || KeybindingType::MinimizeTile);
    engine.register_fn("quit", || KeybindingType::Quit);
    engine.register_fn("toggle_floating_mode", || {
        KeybindingType::ToggleFloatingMode
    });
    engine.register_fn("toggle_work_mode", || KeybindingType::ToggleWorkMode);
    engine.register_fn("toggle_fullscreen", || KeybindingType::ToggleFullscreen);
    engine.register_fn("change_workspace", |id: i32| {
        KeybindingType::ChangeWorkspace(id)
    });
    engine.register_fn("move_to_workspace", |id: i32| {
        KeybindingType::MoveToWorkspace(id)
    });
    engine.register_fn("move_workspace_to_monitor", |id: i32| {
        KeybindingType::MoveWorkspaceToMonitor(id)
    });
    engine.register_fn("toggle_mode", |mode: String| {
        KeybindingType::ToggleMode(mode)
    });
    engine.register_fn("toggle_mode", |mode: String| {
        KeybindingType::ToggleMode(mode)
    });
    engine.register_fn("increment_config", |key: String, value: i32| {
        KeybindingType::IncrementConfig(key, value)
    });
    engine.register_fn("decrement_config", |key: String, value: i32| {
        KeybindingType::DecrementConfig(key, value)
    });
    engine.register_fn("toggle_config", |key: String| {
        KeybindingType::ToggleConfig(key)
    });
    engine.register_fn("launch", |program: String| KeybindingType::Launch(program));
    engine.register_fn("focus", |direction: String| {
        KeybindingType::Focus(Direction::from_str(&direction).unwrap())
    });
    engine.register_fn("swap", |direction: String| {
        KeybindingType::Swap(Direction::from_str(&direction).unwrap())
    });
    engine.register_fn("resize", |direction: String, amount: i32| {
        KeybindingType::Resize(Direction::from_str(&direction).unwrap(), amount)
    });
    engine.register_fn("split", |direction: String| {
        KeybindingType::Split(SplitDirection::from_str(&direction).unwrap())
    });
}
