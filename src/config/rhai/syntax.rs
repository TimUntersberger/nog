use crate::{
    config::Config,
    keybindings::{keybinding::Keybinding, keybinding_type::KeybindingType},
};
use rhai::{Dynamic, Engine};
use std::str::FromStr;

pub fn init(engine: &mut Engine) -> Result<(), Box<dyn std::error::Error>> {
    engine.register_custom_syntax(
        &["bind", "$expr$", "to", "$expr$"], // the custom syntax
        0, // the number of new variables declared within this custom syntax
        |engine, ctx, scope, inputs| {
            let key_expression = inputs.get(0).unwrap();
            let func_expression = inputs.get(1).unwrap();

            let key = engine
                .eval_expression_tree(ctx, scope, key_expression)?
                .as_str()?
                .to_string();

            let binding = engine
                .eval_expression_tree(ctx, scope, func_expression)?
                .cast::<KeybindingType>();

            println!("{:?}", key);
            println!("{:?}", binding);

            let mut kb = Keybinding::from_str(&key).unwrap();

            kb.typ = binding;

            scope.set_value("__new_keybinding", kb);

            engine
                .consume_with_scope(scope, "__keybindings.push(__new_keybinding);")
                .unwrap();

            Ok(().into())
        },
    )?;

    Ok(())
}
