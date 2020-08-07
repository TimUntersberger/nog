use super::win_event_type::WinEventType;

#[derive(Clone, Copy, Debug)]
pub struct WinEvent {
    pub typ: WinEventType,
    pub hwnd: i32,
}
