use rhai::Engine;

use crate::event::EventChannel;

mod popup;

pub fn init(engine: &mut Engine, chan: &EventChannel) {
    popup::init(engine, chan);
}
