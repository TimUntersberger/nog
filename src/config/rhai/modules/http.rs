use crate::{config::rhai::types::http_response::HttpResponse, bar};
use rhai::{ImmutableString, Module, Dynamic, Map};
use std::{thread, collections::HashMap};
use reqwest::blocking::Client;

fn json_to_dynamic(json: serde_json::Value) -> Dynamic {
    match json {
        serde_json::Value::Null => ().into(),
        serde_json::Value::Bool(x) => x.into(),
        serde_json::Value::Number(x) => {
            if x.is_i64() {
                Dynamic::from(x.as_i64().unwrap() as i32)
            } else {
                Dynamic::from(x.as_f64().unwrap())
            }
        }
        serde_json::Value::String(x) => x.into(),
        serde_json::Value::Array(x) => x
            .iter()
            .map(|v| json_to_dynamic(v.clone()))
            .collect::<Vec<Dynamic>>()
            .into(),
        serde_json::Value::Object(x) => x
            .iter()
            .map(|(key, value)| (key.clone().into(), json_to_dynamic(value.clone())))
            .collect::<HashMap<ImmutableString, Dynamic>>()
            .into(),
    }
}

fn http_get(url: ImmutableString, options: Option<Map>) -> HttpResponse {
    let mut response = HttpResponse::default();
    let url = url.to_string();
    let client = Client::new();
    let rb = client.get(&url);

    if let Ok(res) = rb.send() {
        let content_type = res
            .headers()
            .get("content-type")
            .map(|h| h.to_str().unwrap().to_string())
            .unwrap_or_default();
        let body = res.text().unwrap();
        if content_type.contains("application/json") {
            let json = serde_json::from_str::<serde_json::Value>(&body).unwrap();
            let map = json_to_dynamic(json);
            response.body = map;
        } else {
            response.body = body.into();
        }
    }

    response
}

pub fn new() -> Module {
    let mut module = Module::new();

    module.set_fn_1("get", |url: ImmutableString| {
        Ok(http_get(url, None))
    });
    module.set_fn_2("get", |url: ImmutableString, options: Map| {
        Ok(http_get(url, Some(options)))
    });

    module
}
