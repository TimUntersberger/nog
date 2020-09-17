use thiserror::Error;

pub type SpecificError = win::WinError;
pub type NativeWindow = win::WinWindow;
pub type WindowId = i32;

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Failed to show the window")]
    ShowWindow(SpecificError),
    #[error("Failed to hide the window")]
    HideWindow(SpecificError),
    #[error("Failed to focus window")]
    FocusWindow(SpecificError),
    #[error("Failed to close window")]
    CloseWindow(SpecificError),
    #[error("Failed to cleanup window")]
    CleanupWindow(SpecificError),
    #[error("Failed to minimize window")]
    MinimizeWindow(SpecificError),
    #[error("Failed to maximize window")]
    MaximizeWindow(SpecificError),
    #[error("An unknown error occured")]
    Unknown(SpecificError)
}

pub type SystemResult<T = ()> = Result<T, SystemError>;

pub mod win;
