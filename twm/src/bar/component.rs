use crate::{display::Display, system::DisplayId, AppState};
use interpreter::{Dynamic, Function, Interpreter, RuntimeError, RuntimeResult};
use parking_lot::Mutex;
use std::{any::Any, collections::HashMap, fmt::Debug, sync::Arc};

pub mod active_mode;
pub mod current_window;
pub mod date;
pub mod fullscreen_indicator;
pub mod padding;
pub mod split_direction;
pub mod time;
pub mod workspaces;

pub const LOCK_TIMEOUT: u64 = 20;

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
    render_fn: Arc<dyn Fn(DisplayId) -> RuntimeResult<Vec<ComponentText>> + Send + Sync>,
    on_click_fn: Option<
        Arc<
            dyn Fn(DisplayId, Arc<Box<dyn Any + Send + Sync>>, usize) -> RuntimeResult<()>
                + Send
                + Sync,
        >,
    >,
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

impl Component {
    pub fn new(
        name: &str,
        render_fn: impl Fn(DisplayId) -> RuntimeResult<Vec<ComponentText>> + Send + Sync + 'static,
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

        let name = string!(obj
            .get("name")
            .ok_or("A component has to have a name field of type String")?)?;

        let render_fn = obj
            .get("render")
            .ok_or("A component has to have a render field that is a function")?
            .clone();

        let on_click_fn = obj.get("on_click");

        let i2 = i.clone();

        let mut comp = Component::new(name, move |display_id| {
            let f = render_fn.clone().as_fn()?;
            let dynamics = f
                .invoke(&mut i2.lock(), vec![display_id.0.into()])?
                .as_array()?;
            let mut rendered = Vec::new();

            for d in dynamics {
                rendered.push(match d {
                    Dynamic::String(x) => ComponentText::new().with_display_text(x.clone()),
                    Dynamic::Array(x) => {
                        let items = x.lock().unwrap();
                        assert!(items.len() == 3);
                        ComponentText::new()
                            .with_display_text(string!(&items[0])?.clone())
                            .with_foreground_color(*number!(&items[1])?)
                            .with_background_color(*number!(&items[2])?)
                    }
                    x => {
                        return Err(RuntimeError::UnexpectedType {
                            expected: "String | Array".into(),
                            actual: x.type_name(),
                        })
                    }
                })
            }

            Ok(rendered)
        });

        if let Some(f) = on_click_fn {
            let f = f.clone().as_fn()?;
            let i2 = i.clone();
            comp.with_on_click(move |display_id, value, idx| {
                f.invoke(
                    &mut i2.lock(),
                    vec![display_id.0.into(), value.into(), idx.into()],
                )
                .map(|_| {})
            });
        }

        Ok(comp)
    }

    pub fn into_dynamic(&self, state_arc: Arc<Mutex<AppState>>) -> Dynamic {
        let mut fields: HashMap<String, Dynamic> = HashMap::new();

        fields.insert("name".into(), self.name.clone().into());

        let render_fn = self.render_fn.clone();
        fields.insert(
            "render".into(),
            Function::new("render", None, move |_, args| {
                let display_id = *number!(&args[0])?;
                Ok((render_fn)(DisplayId(display_id))?
                    .iter()
                    .map(|x| {
                        if x.foreground_color == 0 && x.background_color == 0 {
                            x.display_text.clone().into()
                        } else {
                            Dynamic::new_array(vec![
                                Dynamic::String(x.display_text.clone()),
                                Dynamic::Number(x.foreground_color),
                                Dynamic::Number(x.background_color),
                            ])
                        }
                    })
                    .collect::<Vec<_>>()
                    .into())
            })
            .into(),
        );

        if let Some(on_click_fn) = self.on_click_fn.as_ref() {
            let f = on_click_fn.clone();
            let state = state_arc.clone();
            fields.insert(
                "on_click".into(),
                Function::new("on_click", None, move |_, args| {
                    let value = rust_value!(&args[0])?.clone();
                    let idx = *number!(&args[1])?;
                    let display_id = state.lock().get_current_display().id;

                    (f)(display_id, value, idx as usize)?;

                    Ok(().into())
                })
                .into(),
            );
        }

        fields.into()
    }

    pub fn on_click(
        &self,
        display_id: DisplayId,
        value: Arc<Box<dyn Any + Send + Sync>>,
        idx: usize,
    ) -> RuntimeResult<()> {
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
        f: impl Fn(DisplayId, Arc<Box<dyn Any + Send + Sync>>, usize) -> RuntimeResult<()>
            + Send
            + Sync
            + 'static,
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
