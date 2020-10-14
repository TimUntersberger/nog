use crate::{
    display::Display, AppState,
};
use std::{fmt::Debug, sync::Arc};

pub mod active_mode;
pub mod current_window;
pub mod date;
pub mod padding;
pub mod time;
pub mod workspaces;

#[derive(Debug, Clone)]
pub enum ComponentText {
    Basic(String),
    /// (fg, bg, text)
    Colored(Option<u32>, Option<u32>, String),
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
            Self::Colored(fg, _, _) => fg.clone(),
        }
    }

    pub fn get_bg(&self) -> Option<u32> {
        match self {
            Self::Basic(_) => None,
            Self::Colored(_, bg, _) => bg.clone(),
        }
    }
}

pub type RenderFn = Arc<dyn Fn(RenderContext) -> Vec<ComponentText> + Send + Sync>;
/// Receives the Component, the display and the idx of ComponentText which got clicked
pub type OnClickFn = Arc<dyn Fn(OnClickContext) -> () + Send + Sync>;

#[derive(Clone)]
pub struct Component {
    pub name: String,
    pub is_clickable: bool,
    render_fn: RenderFn,
    on_click_fn: Option<OnClickFn>,
}

impl Default for Component {
    fn default() -> Self {
        Self {
            name: "Default".into(),
            is_clickable: false,
            render_fn: Arc::new(|_| vec![ComponentText::Basic("Hello World".into())]),
            on_click_fn: None,
        }
    }
}

pub struct RenderContext<'a> {
    pub display: &'a Display,
    pub state: &'a AppState,
}

pub struct OnClickContext<'a> {
    pub display: &'a Display,
    pub state: &'a AppState,
    pub idx: usize,
}

impl Component {
    pub fn new(name: &str, render_fn: RenderFn) -> Self {
        Self {
            name: name.into(),
            is_clickable: false,
            render_fn,
            on_click_fn: None,
        }
    }

    pub fn on_click(&self, display: &Display, state: &AppState, idx: usize) {
        if let Some(f) = self.on_click_fn.clone() {
            f(OnClickContext {
                display,
                state,
                idx,
            });
        }
    }

    pub fn render(&self, display: &Display, state: &AppState) -> Vec<ComponentText> {
        let f = self.render_fn.clone();

        f(RenderContext { display, state })
    }

    pub fn with_on_click(&mut self, f: OnClickFn) -> &mut Self {
        self.is_clickable = true;
        self.on_click_fn = Some(f);
        self
    }
}

impl Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Component(name: {}, clickable: {})",
            self.name, self.is_clickable
        ))
    }
}
