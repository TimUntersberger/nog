use winapi::shared::windef::RECT;

#[derive(Clone)]
pub struct Window {
    pub id: i32,
    pub name: String,
    pub original_style: i32,
    pub original_rect: RECT
}

impl Window {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: String::from(""),
            original_style: 0,
            original_rect: RECT {
                left: 0,
                right: 0,
                top: 0,
                bottom: 0
            }
        }
    }
}