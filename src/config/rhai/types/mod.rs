use http_response::HttpResponse;
use rhai::Engine;

pub mod http_response;

pub fn init(engine: &mut Engine) {
    engine
        .register_type::<HttpResponse>()
        .register_get("body", |x: &mut HttpResponse| x.body.clone())
        .register_get("content_type", |x: &mut HttpResponse| x.content_type)
        .register_get("status_code", |x: &mut HttpResponse| x.status_code);
}
