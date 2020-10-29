use crate::{display::Display, AppState};
use std::{any::Any, fmt::Debug, sync::Arc};

pub mod active_mode;
pub mod current_window;
pub mod date;
pub mod padding;
pub mod time;
pub mod workspaces;

#[derive(Debug, Clone)]
pub struct ComponentText {
    pub display_text: String,
    pub value: Arc<Box<dyn Any + Sync + Send>>,
    pub foreground_color: i32,
    pub background_color: i32,
}

impl ComponentText {
    pub fn new() -> Self {
        Self {
            display_text: "".into(),
            value: Arc::new(Box::new(())),
            foreground_color: 0,
            background_color: 0,
        }
    }
    pub fn with_display_text(mut self, value: String) -> Self {
        self.display_text = value;
        self
    }
    pub fn with_value(mut self, value: impl Any + Send + Sync) -> Self {
        self.value = Arc::new(Box::new(value));
        self
    }
    pub fn with_foreground_color(mut self, value: i32) -> Self {
        self.foreground_color = value;
        self
    }
    pub fn with_background_color(mut self, value: i32) -> Self {
        self.background_color = value;
        self
    }
}

// #[derive(Debug, Clone)]
// pub enum ComponentText {
//     Basic(String),
//     /// (fg, bg, text)
//     Colored(Option<i32>, Option<i32>, String),
// }

// impl ComponentText {
//     pub fn get_text(&self) -> String {
//         match self {
//             Self::Basic(text) => text.into(),
//             Self::Colored(_, _, text) => text.into(),
//         }
//     }

//     pub fn get_fg(&self) -> Option<i32> {
//         match self {
//             Self::Basic(_) => None,
//             Self::Colored(fg, _, _) => fg.clone(),
//         }
//     }

//     pub fn get_bg(&self) -> Option<i32> {
//         match self {
//             Self::Basic(_) => None,
//             Self::Colored(_, bg, _) => bg.clone(),
//         }
//     }
// }

#[derive(Clone)]
pub struct Component {
    pub name: String,
    pub is_clickable: bool,
    render_fn: Arc<dyn Fn(RenderContext) -> Vec<ComponentText> + Send + Sync>,
    on_click_fn: Option<Arc<dyn Fn(OnClickContext) -> () + Send + Sync>>,
}

impl Default for Component {
    fn default() -> Self {
        Self {
            name: "Default".into(),
            is_clickable: false,
            render_fn: Arc::new(|_| vec![]),
            on_click_fn: None,
        }
    }
}

pub struct RenderContext<'a> {
    pub display: &'a Display,
    pub state: &'a AppState,
}

#[derive(Debug)]
pub struct OnClickContext<'a> {
    pub display: &'a Display,
    pub state: &'a AppState,
    pub value: Arc<Box<dyn Any + Send + Sync>>,
    pub idx: usize,
}

impl Component {
    pub fn new(name: &str, render_fn: impl Fn(RenderContext) -> Vec<ComponentText> + Send + Sync + 'static) -> Self {
        Self {
            name: name.into(),
            is_clickable: false,
            render_fn: Arc::new(render_fn),
            on_click_fn: None,
        }
    }

    pub fn on_click(&self, display: &Display, state: &AppState, value: Arc<Box<dyn Any + Send + Sync>>, idx: usize) {
        if let Some(f) = self.on_click_fn.clone() {
            f(OnClickContext {
                display,
                state,
                value,
                idx,
            });
        }
    }

    pub fn render(&self, display: &Display, state: &AppState) -> Vec<ComponentText> {
        let f = self.render_fn.clone();

        f(RenderContext { display, state })
    }

    pub fn with_on_click(&mut self, f: impl Fn(OnClickContext) -> () + Send + Sync + 'static) -> &mut Self {
        self.is_clickable = true;
        self.on_click_fn = Some(Arc::new(f));
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
