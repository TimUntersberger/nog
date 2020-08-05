use crate::{config::Config, keybindings::keybinding_type::KeybindingType};
use rhai::Engine;

pub fn init(config: &mut Config, engine: &mut Engine) -> Result<(), Box<dyn std::error::Error>> {
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

            // let kb = match binding {
            //     KeybindingType::Launch(program) => Keybinding()
            // }

            Ok(().into())
        },
    )?;

    Ok(())
}
