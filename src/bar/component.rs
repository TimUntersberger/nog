use super::RedrawReason;

pub mod date;
pub mod mode;
pub mod time;

pub enum ComponentText {
    Basic(String),
    /// (fg, bg, text)
    Colored(u32, u32, String),
}

impl ComponentText {
    pub fn get_text(&self) -> String {
        match self {
            Self::Basic(text) => text.into(),
            Self::Colored(_, _, text) => text.into(),
        }
    }

    pub fn get_fg(&self) -> Option<u32> {
        match self {
            Self::Basic(text) => None,
            Self::Colored(fg, _, _) => Some(*fg),
        }
    }

    pub fn get_bg(&self) -> Option<u32> {
        match self {
            Self::Basic(text) => None,
            Self::Colored(_, bg, _) => Some(*bg),
        }
    }
}

pub trait Component {
    fn get_width(&self) -> Option<i32>;
    fn render(&self) -> ComponentText;
    fn should_render(&self, reason: RedrawReason) -> bool;
}
