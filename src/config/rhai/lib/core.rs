use crate::{
    config::rhai::engine,
    event::Event,
    event::EventChannel,
    popup::{Popup, PopupAction},
    keybindings::keybinding::Keybinding,
    keybindings::keybinding_type::KeybindingType,
    AppState,
};

use parking_lot::Mutex;
use rhai::{Array, Dynamic, Engine, FnPtr, Map, RegisterFn};
use std::{sync::Arc, str::FromStr};

pub fn init(engine: &mut Engine, state_arc: Arc<Mutex<AppState>>) {
    let state = state_arc.clone();
    engine.register_fn("nog_get_split_direction", move || -> Dynamic {
        match state
            .lock()
            .get_current_grid()
            .and_then(|g| g.get_focused_tile())
            .map(|t| t.split_direction.to_string())
        {
            Some(s) => s.into(),
            None => ().into(),
        }
    });
}
