use crate::bar::RedrawReason;
use crate::{keybindings::keybinding::Keybinding, win_event_handler::win_event::WinEvent};
use crossbeam_channel::unbounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

#[derive(Debug, Clone)]
pub enum Event {
    Keybinding(Keybinding),
    WinEvent(WinEvent),
    RedrawAppBar(RedrawReason),
    ReloadConfig,
    Exit,
}

pub type EventSender = Sender<Event>;
pub type EventReceiver = Receiver<Event>;

pub struct EventChannel {
    pub sender: EventSender,
    pub receiver: EventReceiver,
}

impl Default for EventChannel {
    fn default() -> Self {
        let (sender, receiver) = unbounded();

        Self { sender, receiver }
    }
}
