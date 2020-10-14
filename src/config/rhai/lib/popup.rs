use crate::{
    config::rhai::engine,
    event::Event,
    event::EventChannel,
    popup::{Popup, PopupAction},
    AppState,
};

use parking_lot::Mutex;
use rhai::{Array, Engine, FnPtr, Map};
use std::sync::Arc;

pub fn init(engine: &mut Engine, state: Arc<Mutex<AppState>>) {
    #[allow(deprecated)]
    engine.register_raw_fn(
        "popup_new",
        &[std::any::TypeId::of::<Map>()],
        move |_engine, _module, args| {
            let options = args[0].clone().cast::<Map>();
            let mut p = Popup::new();
            let sender = state.lock().event_channel.sender.clone();

            for (key, val) in options {
                p = match key.as_str() {
                    "text" => p.with_text(
                        val.cast::<Array>()
                            .iter()
                            .map(|v| v.as_str().unwrap())
                            .collect::<Vec<&str>>()
                            .as_slice(),
                    ),
                    "padding" => p.with_padding(val.as_int().unwrap()),
                    "actions" => {
                        let actions = val.cast::<Array>();

                        for action in actions {
                            let settings = action.cast::<Map>();
                            let mut action = PopupAction::default();

                            for (key, val) in settings {
                                match key.as_str() {
                                    "text" => {
                                        action.text = val.to_string();
                                    }
                                    "cb" => {
                                        let fp = val.cast::<FnPtr>();
                                        let idx = engine::add_callback(fp);
                                        action.cb = Some(Arc::new(move || engine::call(idx)));
                                    }
                                    _ => {}
                                };
                            }

                            p.actions.push(action);
                        }

                        p
                    }
                    _ => p,
                };
            }

            sender.send(Event::NewPopup(p));

            Ok(())
        },
    );
}
