use crate::{display::Display, AppState};
use interpreter::{Dynamic, Interpreter, RuntimeError, RuntimeResult};
use parking_lot::Mutex;
use std::{any::Any, fmt::Debug, sync::Arc};

pub mod active_mode;
pub mod current_window;
pub mod date;
pub mod padding;
pub mod split_direction;
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

#[derive(Clone)]
pub struct Component {
    pub name: String,
    pub is_clickable: bool,
    render_fn: Arc<dyn Fn(RenderContext) -> RuntimeResult<Vec<ComponentText>> + Send + Sync>,
    on_click_fn: Option<Arc<dyn Fn(OnClickContext) -> RuntimeResult<()> + Send + Sync>>,
}

impl Default for Component {
    fn default() -> Self {
        Self {
            name: "Default".into(),
            is_clickable: false,
            render_fn: Arc::new(|_| Ok(vec![])),
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
    pub fn new(
        name: &str,
        render_fn: impl Fn(RenderContext) -> RuntimeResult<Vec<ComponentText>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            is_clickable: false,
            render_fn: Arc::new(render_fn),
            on_click_fn: None,
        }
    }

    pub fn from_dynamic(i: Arc<Mutex<Interpreter>>, d: Dynamic) -> RuntimeResult<Self> {
        let obj_ref = object!(d)?;
        let obj = obj_ref.lock().unwrap();
        let name = string!(obj.get("name").ok_or(
            "nog.bar.register_component requires the object to have a name field of type String"
        )?)?;

        let render_fn = obj.get("render").ok_or("nog.bar.register_component requires the object to have a render field that is a function")?.clone();
        let on_click_fn = obj.get("on_click");

        let i2 = i.clone();

        let mut comp = Component::new(name, move |_| {
            let f = render_fn.clone().as_fn()?;

            Ok(f.invoke(&mut i2.lock(), vec![])?
                .as_array()?
                .iter()
                .map(|d| match d {
                    Dynamic::String(x) => ComponentText::new().with_display_text(x.clone()),
                    x => unreachable!(x),
                })
                .collect())
        });

        if let Some(f) = on_click_fn {
            let f = f.clone().as_fn()?;
            let i2 = i.clone();
            comp.with_on_click(move |_| f.invoke(&mut i2.lock(), vec![]).map(|_| {}));
        }

        Ok(comp)
    }

    pub fn on_click(
        &self,
        display: &Display,
        state: &AppState,
        value: Arc<Box<dyn Any + Send + Sync>>,
        idx: usize,
    ) -> RuntimeResult<()> {
        if let Some(f) = self.on_click_fn.clone() {
            f(OnClickContext {
                display,
                state,
                value,
                idx,
            })?;
        }

        Ok(())
    }

    pub fn render(&self, display: &Display, state: &AppState) -> RuntimeResult<Vec<ComponentText>> {
        let f = self.render_fn.clone();

        f(RenderContext { display, state })
    }

    pub fn with_on_click(
        &mut self,
        f: impl Fn(OnClickContext) -> RuntimeResult<()> + Send + Sync + 'static,
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
