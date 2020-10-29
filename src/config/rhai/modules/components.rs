use crate::{
    bar,
    bar::component::{Component, ComponentText},
    config::rhai::engine::AST,
    config::rhai::engine::ENGINE,
    config::rhai::engine::SCOPE,
    display::Display,
};
use log::error;
use rhai::{Array, Dynamic, FnPtr, ImmutableString, Map, Module};

fn create_component(name: ImmutableString, render_fn: FnPtr, options: Map) -> Component {
    let render_fn_name = render_fn.fn_name().to_string();
    let mut component = Component::new(
        &name,
        move |_| {
            let engine = ENGINE.lock();
            let mut scope = SCOPE.lock();
            let ast = AST.lock();
            let result = engine
                .call_fn::<(), Vec<Dynamic>>(&mut *scope, &*ast, &render_fn_name, ())
                .map_err(|e| error!("{}", e.to_string()))
                .unwrap_or_default();

            let return_value = result
                .iter()
                .map(|x| match x.type_name() {
                    "string" => Some(
                        ComponentText::new().with_display_text(x.as_str().unwrap().to_string()),
                    ),
                    "array" => {
                        let tuple = x.clone().cast::<Array>();
                        let fg = tuple.get(0).unwrap().as_int().unwrap();
                        let bg = tuple.get(1).unwrap().as_int().unwrap();
                        let text = tuple.get(2).unwrap().as_str().unwrap().to_string();

                        Some(
                            ComponentText::new()
                                .with_display_text(text)
                                .with_foreground_color(fg)
                                .with_background_color(bg),
                        )
                    }
                    _ => None,
                })
                .filter(|x| x.is_some())
                .map(|x| x.unwrap())
                .collect();

            return_value
        },
    );

    for (key, val) in options.iter() {
        match key.as_str() {
            "on_click" => {
                let f = val.clone().cast::<FnPtr>();
                let fn_name = f.fn_name().to_string();

                component.with_on_click(move |ctx| {
                    let engine = ENGINE.lock();
                    let mut scope = SCOPE.lock();
                    let ast = AST.lock();
                    let _ = engine
                        .call_fn::<(Display, usize), ()>(
                            &mut *scope,
                            &*ast,
                            &fn_name,
                            (ctx.display.clone(), ctx.idx),
                        )
                        .map_err(|e| error!("{}", e.to_string()));
                });
            }
            _ => {}
        }
    }

    component
}

pub fn new() -> Module {
    let mut module = Module::new();

    module.set_fn_0("active_mode", || Ok(bar::component::active_mode::create()));
    module.set_fn_0("current_window", || {
        Ok(bar::component::current_window::create())
    });
    module.set_fn_0("workspaces", || Ok(bar::component::workspaces::create()));
    module.set_fn_1("date", |pattern: ImmutableString| {
        Ok(bar::component::date::create(pattern.to_string()))
    });
    module.set_fn_1("time", |pattern: ImmutableString| {
        Ok(bar::component::time::create(pattern.to_string()))
    });
    module.set_fn_1("padding", |amount: i32| {
        Ok(bar::component::padding::create(amount))
    });

    module.set_fn_2("create", |name: ImmutableString, render_fn: FnPtr| {
        Ok(create_component(name, render_fn, Map::new()))
    });
    module.set_fn_3(
        "create",
        |name: ImmutableString, render_fn: FnPtr, options: Map| {
            Ok(create_component(name, render_fn, options))
        },
    );

    module
}
