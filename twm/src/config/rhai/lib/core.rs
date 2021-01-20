use crate::{
    config::rhai::engine,
    event::Event,
    event::EventChannel,
    keybindings::keybinding::Keybinding,
    keybindings::keybinding_type::KeybindingType,
    popup::{Popup, PopupAction},
    system::api,
    system::NativeWindow,
    AppState,
};

use log::error;
use parking_lot::Mutex;
use rhai::{Array, Dynamic, Engine, FnPtr, Map, RegisterFn};
use std::{str::FromStr, sync::Arc};

pub fn init(engine: &mut Engine, state_arc: Arc<Mutex<AppState>>) {
    let state = state_arc.clone();
    engine.register_fn("nog_get_split_direction", move || -> Dynamic {
        match state
            .lock()
            .get_current_grid()
            .map(|g| g.next_axis.to_string())
        {
            Some(s) => s.into(),
            None => ().into(),
        }
    });

    engine.register_fn("nog_get_focused_window_info", move || -> Dynamic {
        match NativeWindow::get_foreground_window() {
            Ok(window) => {
                let mut info = rhai::Map::new();
                info.insert("process_name".into(), window.get_process_name().into());
                info.insert(
                    "title".into(),
                    window.get_title().map(|s| s.into()).unwrap_or(().into()),
                );
                info.into()
            }
            Err(e) => {
                error!("{:?}", e);
                ().into()
            }
        }
    });
}
