use crate::hot_key_manager::Keybinding;
use crate::win_event_handler::WinEvent;
use crate::app_bar::RedrawAppBarReason;
use crossbeam_channel::unbounded;
use crossbeam_channel::Sender;
use crossbeam_channel::Receiver;

#[derive(Debug)]
pub enum Event {
    Keybinding(Keybinding),
    WinEvent(WinEvent),
    RedrawAppBar(RedrawAppBarReason),
    Exit
}

pub type EventSender = Sender<Event>;
pub type EventReceiver = Receiver<Event>;

pub struct EventChannel {
    pub sender: EventSender,
    pub receiver: EventReceiver
}

impl Default for EventChannel {
    fn default() -> Self {
        let (sender, receiver) = unbounded();

        Self {
            sender,
            receiver
        }
    }
}