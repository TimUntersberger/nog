use thiserror::Error;

pub mod win;

pub use win::api;
pub use win::win_event_listener::WinEventListener;
pub use win::Window as NativeWindow;
pub use win::BIN_NAME as BIN_NAME;

pub type SpecificError = win::WinError;

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct WindowId(i32);

impl std::fmt::Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Into<i32> for WindowId {
    fn into(self) -> i32 {
        self.0
    }
}

impl From<i32> for WindowId {
    fn from(v: i32) -> Self {
        Self(v)
    }
}

impl PartialEq<i32> for WindowId {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

#[derive(Clone, Copy, Debug, Default, PartialOrd, PartialEq, Hash, Eq)]
pub struct DisplayId(i32);

impl PartialEq<i32> for DisplayId {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Rectangle {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

impl Rectangle {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Failed to show window")]
    ShowWindow(SpecificError),
    #[error("Failed to initialize window")]
    Init(SpecificError),
    #[error("Failed to hide window")]
    HideWindow(SpecificError),
    #[error("Failed to focus window")]
    FocusWindow(SpecificError),
    #[error("Failed to redraw window")]
    RedrawWindow(SpecificError),
    #[error("Failed to close window")]
    CloseWindow(SpecificError),
    #[error("Failed to cleanup window")]
    CleanupWindow(SpecificError),
    #[error("Failed to minimize window")]
    MinimizeWindow(SpecificError),
    #[error("Failed to maximize window")]
    MaximizeWindow(SpecificError),
    #[error("Failed to draw tile")]
    DrawTile(SpecificError),
    #[error("An unknown error occured")]
    Unknown(SpecificError),
}

pub type SystemResult<T = ()> = Result<T, SystemError>;
