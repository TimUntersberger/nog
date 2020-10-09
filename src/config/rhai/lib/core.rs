use crate::{
    config::rhai::engine,
    event::Event,
    event::EventChannel,
    popup::{Popup, PopupAction},
    AppState,
};

use parking_lot::Mutex;
use rhai::{Array, Engine, FnPtr, Map, RegisterFn};
use std::sync::Arc;

pub fn init(engine: &mut Engine, state: Arc<Mutex<AppState>>) {
    engine.register_fn("nog_get_split_direction", move || {
        state
            .lock()
            .get_current_grid()
            .and_then(|g| g.get_focused_tile())
            .map(|t| t.split_direction)
    });
}
