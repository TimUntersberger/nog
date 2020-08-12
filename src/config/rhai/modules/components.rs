use crate::bar;
use rhai::{ImmutableString, Module};

pub fn new() -> Module {
    let mut module = Module::new();

    module.set_fn_0("active_mode", || Ok(bar::component::active_mode::create()));
    module.set_fn_0("workspaces", || Ok(bar::component::workspaces::create()));
    module.set_fn_1("date", |pattern: ImmutableString| Ok(bar::component::date::create(pattern.to_string())));
    module.set_fn_1("time", |pattern: ImmutableString| Ok(bar::component::time::create(pattern.to_string())));
    module.set_fn_1("padding", |amount: i32| {
        Ok(bar::component::padding::create(amount))
    });

    module
}
