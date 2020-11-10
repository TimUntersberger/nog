use super::engine;
use crate::{direction::Direction, keybindings::action::Action, split_direction::SplitDirection};
use rhai::{Engine, FnPtr, RegisterFn};
use std::str::FromStr;

pub fn init(engine: &mut Engine) {
    engine.register_fn("callback", |fp: FnPtr| {
        Action::Callback(engine::add_callback(fp))
    });
    engine.register_fn("close_tile", || Action::CloseTile);
    engine.register_fn("install_plugins", || Action::InstallPlugins);
    engine.register_fn("update_plugins", || Action::UpdatePlugins);
    engine.register_fn("purge_plugins", || Action::PurgePlugins);
    engine.register_fn("ignore_tile", || Action::IgnoreTile);
    engine.register_fn("minimize_tile", || Action::MinimizeTile);
    engine.register_fn("reset_row", || Action::ResetRow);
    engine.register_fn("reset_column", || Action::ResetColumn);
    engine.register_fn("quit", || Action::Quit);
    engine.register_fn("toggle_floating_mode", || Action::ToggleFloatingMode);
    engine.register_fn("toggle_work_mode", || Action::ToggleWorkMode);
    engine.register_fn("toggle_fullscreen", || Action::ToggleFullscreen);
    engine.register_fn("change_workspace", |id: i32| Action::ChangeWorkspace(id));
    engine.register_fn("move_to_workspace", |id: i32| Action::MoveToWorkspace(id));
    engine.register_fn("move_workspace_to_monitor", |id: i32| {
        Action::MoveWorkspaceToMonitor(id)
    });
    engine.register_fn("toggle_mode", |mode: String| Action::ToggleMode(mode));
    engine.register_fn("increment_config", |key: String, value: i32| {
        Action::IncrementConfig(key, value)
    });
    engine.register_fn("decrement_config", |key: String, value: i32| {
        Action::DecrementConfig(key, value)
    });
    engine.register_fn("toggle_config", |key: String| Action::ToggleConfig(key));
    engine.register_fn("launch", |program: String| Action::Launch(program));
    engine.register_fn("focus", |direction: String| {
        Action::Focus(Direction::from_str(&direction).unwrap())
    });
    engine.register_fn("swap", |direction: String| {
        Action::Swap(Direction::from_str(&direction).unwrap())
    });
    engine.register_fn("resize", |direction: String, amount: i32| {
        Action::Resize(Direction::from_str(&direction).unwrap(), amount)
    });
    engine.register_fn("split", |direction: String| {
        Action::Split(SplitDirection::from_str(&direction).unwrap())
    });
}
