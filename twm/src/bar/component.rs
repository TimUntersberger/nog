use crate::{display::Display, system::DisplayId, AppState};
use mlua::Result as RuntimeResult;
use parking_lot::Mutex;
use std::{any::Any, collections::HashMap, fmt::Debug, sync::Arc};

// pub mod active_mode;
// pub mod current_window;
// pub mod date;
// pub mod fullscreen_indicator;
// pub mod padding;
// pub mod split_direction;
// pub mod time;
// pub mod workspaces;

pub const LOCK_TIMEOUT: u64 = 20;

#[derive(Debug, Clone)]
pub struct ComponentText {
    pub display_text: String,
    pub value: i32,
    pub foreground_color: i32,
    pub background_color: i32,
}

impl ComponentText {
    pub fn new() -> Self {
        Self {
            display_text: "".into(),
            value: 0,
            foreground_color: 0,
            background_color: 0,
        }
    }
    pub fn with_display_text(mut self, value: String) -> Self {
        self.display_text = value;
        self
    }
    pub fn with_value(mut self, value: i32) -> Self {
        self.value = value;
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

#[derive(Clone)]
pub struct Component {
    pub name: String,
    pub is_clickable: bool,
    pub lua_render_id: Option<usize>,
    pub lua_on_click_id: Option<usize>,
    render_fn: Arc<dyn for<'a> Fn(DisplayId) -> RuntimeResult<Vec<ComponentText>> + Send + Sync>,
    on_click_fn: Option<Arc<dyn Fn(DisplayId, i32, usize) -> RuntimeResult<()> + Send + Sync>>,
}

impl Default for Component {
    fn default() -> Self {
        Self {
            name: "Default".into(),
            is_clickable: false,
            lua_render_id: None,
            lua_on_click_id: None,
            render_fn: Arc::new(|_| Ok(vec![])),
            on_click_fn: None,
        }
    }
}

impl Component {
    pub fn new(
        name: &str,
        render_fn: impl Fn(DisplayId) -> RuntimeResult<Vec<ComponentText>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            is_clickable: false,
            lua_render_id: None,
            lua_on_click_id: None,
            render_fn: Arc::new(render_fn),
            on_click_fn: None,
        }
    }

    pub fn on_click(&self, display_id: DisplayId, value: i32, idx: usize) -> RuntimeResult<()> {
        if let Some(f) = self.on_click_fn.clone() {
            f(display_id, value, idx)?;
        }

        Ok(())
    }

    pub fn render(&self, display_id: DisplayId) -> RuntimeResult<Vec<ComponentText>> {
        let f = self.render_fn.clone();

        f(display_id)
    }

    pub fn with_on_click(
        &mut self,
        f: impl Fn(DisplayId, i32, usize) -> RuntimeResult<()> + Send + Sync + 'static,
    ) -> &mut Self {
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
