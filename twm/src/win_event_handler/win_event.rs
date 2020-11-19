use crate::system::NativeWindow;

use super::win_event_type::WinEventType;

#[derive(Clone, Debug)]
pub struct WinEvent {
    pub typ: WinEventType,
    pub window: NativeWindow,
}
