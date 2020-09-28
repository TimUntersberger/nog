use crate::{config::Config, display::Display, keybindings::KbManager, tile_grid::TileGrid};
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
pub type OnClickFn = Arc<dyn Fn(&Component, &Display, usize) -> () + Send + Sync>;

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
    display: &'a Display,
    workspace_id: i32,
    grids: &'a Vec<TileGrid>,
    kb_manager: &'a KbManager,
    config: &'a Config,
}

impl<'a> RenderContext<'a> {
    pub fn get_current_grid(&self) -> &TileGrid {
        self.grids
            .iter()
            .find(|g| g.id == self.workspace_id)
            .unwrap()
    }
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

    pub fn on_click(&self, display: &Display, idx: usize) {
        if let Some(fun) = self.on_click_fn.clone() {
            fun(self, display, idx);
        }
    }

    pub fn render(
        &self,
        kb_manager: &KbManager,
        display: &Display,
        grids: &Vec<TileGrid>,
        workspace_id: i32,
        config: &Config,
    ) -> Vec<ComponentText> {
        let f = self.render_fn.clone();

        f(RenderContext {
            display,
            kb_manager,
            grids,
            workspace_id,
            config,
        })
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
