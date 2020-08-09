use crate::display::Display;
use std::sync::Arc;

pub mod date;
pub mod mode;
pub mod time;
pub mod workspaces;
pub mod padding;

#[derive(Debug)]
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
            Self::Basic(_) => None,
            Self::Colored(fg, _, _) => Some(*fg),
        }
    }

    pub fn get_bg(&self) -> Option<u32> {
        match self {
            Self::Basic(_) => None,
            Self::Colored(_, bg, _) => Some(*bg),
        }
    }
}

pub type RenderFn = Arc<dyn Fn(&Component, &Display) -> Vec<ComponentText> + Send + Sync>;
pub type OnClickFn = Arc<dyn Fn(&Component) -> () + Send + Sync>;

#[derive(Clone)]
pub struct Component {
    pub is_clickable: bool,
    render_fn: RenderFn,
    on_click_fn: Option<OnClickFn>,
}

impl Component {
    pub fn new(render_fn: RenderFn) -> Self {
        Self {
            is_clickable: false,
            render_fn,
            on_click_fn: None,
        }
    }

    pub fn on_click(&self) {
        if let Some(fun) = self.on_click_fn.clone() {
            fun(self);
        }
    }

    pub fn render(&self, display: &Display) -> Vec<ComponentText> {
        let f = self.render_fn.clone();

        f(self, display)
    }

    pub fn with_on_click(&mut self, f: OnClickFn) -> &mut Self {
        self.is_clickable = true;
        self.on_click_fn = Some(f);
        self
    }
}