use crate::config::rhai::types::http_response::HttpResponse;
use reqwest::{
    blocking::{Client, RequestBuilder},
    Method,
};
use rhai::{
    de::from_dynamic, Array, Dynamic, EvalAltResult, ImmutableString, Map, Module, FLOAT, INT,
};

fn load_request_body(rb: RequestBuilder, body: Dynamic) -> RequestBuilder {
    match body.type_name() {
        "string" => rb.body(body.as_str().unwrap().to_string()),
        "int" => rb.body(body.as_int().unwrap().to_string()),
        "float" => rb.body(body.as_float().unwrap().to_string()),
        "bool" => rb.body(body.as_bool().unwrap().to_string()),
        "char" => rb.body(body.as_char().unwrap().to_string()),
        "()" => rb,
        _ => rb.json(&from_dynamic::<serde_json::Value>(&body).unwrap()),
    }
}

fn request<T>(
    method: Method,
    url: ImmutableString,
    body: T,
) -> Result<HttpResponse, Box<EvalAltResult>>
where
    T: Into<Dynamic>,
{
    let body = body.into();
    let url = url.to_string();
    let client = Client::new();
    let mut rb = client.request(method, &url);
    rb = load_request_body(rb, body);

    Ok(HttpResponse::from_res(rb.send().unwrap()))
}

macro_rules! register_methods {
    ($module: ident, $name: tt, $method: path) => {
        $module.set_fn_1($name, move |url| request($method, url, ()));
        $module.set_fn_2($name, move |url, body: Map| request($method, url, body));
        $module.set_fn_2($name, move |url, body: String| request($method, url, body));
        $module.set_fn_2($name, move |url, body: INT| request($method, url, body));
        $module.set_fn_2($name, move |url, body: FLOAT| request($method, url, body));
        $module.set_fn_2($name, move |url, body: Array| request($method, url, body));
    };
}

pub fn new() -> Module {
    let mut module = Module::new();

    register_methods!(module, "put", Method::PUT);
    register_methods!(module, "patch", Method::PATCH);
    register_methods!(module, "delete", Method::DELETE);
    register_methods!(module, "get", Method::GET);
    register_methods!(module, "post", Method::POST);

    module
}
