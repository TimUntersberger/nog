use crate::{
    config::rhai::engine::{self, AST, ENGINE, SCOPE},
    popup::{Popup, PopupAction},
    DISPLAYS,
};
use log::error;
use rhai::{Array, Engine, FnPtr, Func, Map, RegisterFn, Scope};
use std::sync::Arc;

pub fn init(engine: &mut Engine) {
    #[allow(deprecated)]
    engine.register_raw_fn(
        "popup_new",
        &[std::any::TypeId::of::<Map>()],
        move |engine, module, args| {
            let options = args[0].clone().cast::<Map>();
            let mut p = Popup::new();

            for (key, val) in options {
                match key.as_str() {
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
                                        let name = val.cast::<FnPtr>().fn_name().to_string();
                                        action.cb = Some(Arc::new(move || engine::call(&name)));
                                    }
                                    _ => {}
                                };
                            }

                            p.actions.push(action);
                        }
                        &mut p
                    }
                    _ => &mut p,
                };
            }

            std::thread::spawn(move || {
                loop {
                    std::thread::sleep_ms(20);
                    if DISPLAYS.lock().unwrap().len() > 0 {
                        break;
                    }
                }
                p.create();
            });

            Ok(())
        },
    );
}
