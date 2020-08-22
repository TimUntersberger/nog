use rhai::Engine;
use http_response::HttpResponse;

pub mod http_response;

pub fn init(engine: &mut Engine) {
    engine.register_type::<HttpResponse>()
        .register_get("body", |x: &mut HttpResponse| x.body.clone());
}