use rhai::{ser::to_dynamic, Dynamic};

#[derive(Copy, Clone)]
pub enum ContentType {
    Json,
    Text,
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType::Text
    }
}

#[derive(Default, Clone)]
pub struct HttpResponse {
    pub body: Dynamic,
    pub status_code: i32,
    pub content_type: ContentType,
}

impl HttpResponse {
    pub fn from_res(res: reqwest::blocking::Response) -> Self {
        let mut this = Self::default();

        this.status_code = res.status().as_u16() as i32;

        let content_type = res
            .headers()
            .get("content-type")
            .map(|h| h.to_str().unwrap().to_string())
            .unwrap_or_default();

        let body = res.text().unwrap();
        if content_type.contains("application/json") {
            let json = serde_json::from_str::<serde_json::Value>(&body).unwrap();
            let map = to_dynamic(json).unwrap();
            this.body = map;
            this.content_type = ContentType::Json;
        } else {
            this.body = body.into();
        };

        this
    }
}
