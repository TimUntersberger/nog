use crate::{split_direction::SplitDirection, direction::Direction};

pub type Command = String;
#[derive(Display, Debug, Clone, PartialEq)]
pub enum KeybindingType {
    CloseTile,
    Quit,
    ChangeWorkspace(i32),
    ToggleFloatingMode,
    ToggleWorkMode,
    IncrementConfig(String, i32),
    DecrementConfig(String, i32),
    ToggleConfig(String),
    MoveWorkspaceToMonitor(i32),
    ToggleFullscreen,
    Launch(Command),
    Focus(Direction),
    Swap(Direction),
    MoveToWorkspace(i32),
    Split(SplitDirection),
}

