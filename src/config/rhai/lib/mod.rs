use rhai::Engine;

use crate::event::{EventChannel, EventSender};

mod popup;

pub fn init(engine: &mut Engine, sender: EventSender) {
    popup::init(engine, sender);
}
