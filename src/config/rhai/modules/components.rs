use crate::bar;
use rhai::Module;

pub fn new() -> Module {
    let mut module = Module::new();

    module.set_fn_0("date", || Ok(bar::component::date::create()));
    module.set_fn_0("active_mode", || Ok(bar::component::active_mode::create()));
    module.set_fn_1("padding", |amount: i32| {
        Ok(bar::component::padding::create(amount))
    });
    module.set_fn_0("time", || Ok(bar::component::time::create()));
    module.set_fn_0("workspaces", || Ok(bar::component::workspaces::create()));

    module
}
