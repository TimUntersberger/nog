use reqwest::blocking::Client;
use rhai::{Engine, ImmutableString};
use std::thread;
use crate::config::rhai::engine::ENGINE;

pub fn init(engine: &mut Engine) {
    #[allow(deprecated)]
    engine.register_raw_fn(
        "http_get",
        &[
            std::any::TypeId::of::<ImmutableString>(),
        ],
        |_, _, args| {
            let url = args.get(0).unwrap().as_str().unwrap().to_string();
            thread::spawn(move || {
                let client = Client::new();
                let rb = client.get(&url);
                if let Ok(res) = rb.send() {
                    println!("{}", res.status());
                    let engine = ENGINE.lock().unwrap();
                    let body = res.text().unwrap();
                    let str = r#"{
                        "test": [{ "str": "str" }]
                    }"#;
                    let map = engine.parse_json(str, true);
                    dbg!(map);
                }
            });

            Ok(())
        },
    );
}
