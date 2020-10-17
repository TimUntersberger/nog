use std::sync::Arc;

use parking_lot::Mutex;
use rhai::Engine;

use crate::AppState;

mod core;
mod popup;

pub fn init(engine: &mut Engine, state_arc: Arc<Mutex<AppState>>) {
    popup::init(engine, state_arc.clone());
    core::init(engine, state_arc.clone());
}
