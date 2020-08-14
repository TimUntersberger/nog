#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BarConfig {
    pub height: i32,
    pub color: i32,
    pub font: String,
    pub font_size: i32
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            height: 20,
            color: 0x2e3440,
            font: "Consolas".into(),
            font_size: 18
        }
    }
}
