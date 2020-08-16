use crate::{direction::Direction, split_direction::SplitDirection};

pub type Command = String;
#[derive(Display, Clone, PartialEq, Debug)]
pub enum KeybindingType {
    CloseTile,
    MinimizeTile,
    ResetColumn,
    ResetRow,
    Quit,
    ChangeWorkspace(i32),
    ToggleFloatingMode,
    ToggleMode(String),
    ToggleWorkMode,
    IncrementConfig(String, i32),
    DecrementConfig(String, i32),
    ToggleConfig(String),
    MoveWorkspaceToMonitor(i32),
    ToggleFullscreen,
    Launch(Command),
    Focus(Direction),
    Resize(Direction, i32),
    Swap(Direction),
    Callback(String),
    MoveToWorkspace(i32),
    Split(SplitDirection),
}
