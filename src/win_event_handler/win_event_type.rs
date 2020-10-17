#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WinEventType {
    Destroy,
    Hide,
    ///Takes a bool, which tells us whether to ignore all rules
    Show(bool),
    FocusChange,
}

#[cfg(target_os = "windows")]
use winapi::um::winuser::{
    EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND,
};
#[cfg(target_os = "windows")]
impl WinEventType {
    pub fn from_u32(v: u32) -> Option<Self> {
        if v == EVENT_OBJECT_DESTROY {
            Some(Self::Destroy)
        } else if v == EVENT_OBJECT_SHOW {
            Some(Self::Show(false))
        } else if v == EVENT_SYSTEM_FOREGROUND {
            Some(Self::FocusChange)
        } else if v == EVENT_OBJECT_HIDE {
            Some(Self::Hide)
        } else {
            None
        }
    }
}
