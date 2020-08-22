use rhai::Dynamic;

#[derive(Default, Clone)]
pub struct HttpResponse {
    pub body: Dynamic
}