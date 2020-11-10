use crate::{system::NativeWindow, AppState};
use log::error;
use parking_lot::Mutex;
use rhai::{Dynamic, Engine, RegisterFn};
use std::sync::Arc;

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
