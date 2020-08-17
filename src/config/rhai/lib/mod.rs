use rhai::Engine;

mod popup;
mod http;

pub fn init(engine: &mut Engine) {
    popup::init(engine);
    http::init(engine);
}
