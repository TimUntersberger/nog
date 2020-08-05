use crate::keybindings::{keybinding::Keybinding, keybinding_type::KeybindingType};
use rhai::{Scope, Engine};
use std::str::FromStr;

fn add_keybinding(engine: &Engine, scope: &mut Scope, key: String, binding: KeybindingType) {
    println!("{:?}", key);
    println!("{:?}", binding);

    let mut kb = Keybinding::from_str(&key).unwrap();

    kb.typ = binding;

    scope.set_value("__new_keybinding", kb);

    engine
        .consume_with_scope(scope, "__keybindings.push(__new_keybinding);")
        .unwrap();
}

pub fn init(engine: &mut Engine) -> Result<(), Box<dyn std::error::Error>> {
    engine.register_custom_syntax(
        &["bind", "$expr$", "$expr$"], // the custom syntax
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

            add_keybinding(engine, scope, key, binding);

            Ok(().into())
        },
    )?;

    engine.register_custom_syntax(
        &["bind_range", "$expr$", "$expr$", "$expr$", "$ident$"], // the custom syntax
        0, // the number of new variables declared within this custom syntax
        |engine, ctx, scope, inputs| {
            let from_expression = inputs.get(0).unwrap();
            let to_expression = inputs.get(1).unwrap();
            let mod_expression = inputs.get(2).unwrap();
            let func_expression = inputs.get(3).unwrap();

            let from = engine
                .eval_expression_tree(ctx, scope, from_expression)?
                .as_int()?;

            let to = engine
                .eval_expression_tree(ctx, scope, to_expression)?
                .as_int()?;

            let modifier = engine
                .eval_expression_tree(ctx, scope, mod_expression)?
                .as_str()?
                .to_string();

            let binding_name = func_expression.get_variable_name().unwrap();

            for i in from..to+1 {
                if i == 11 {
                    break;
                }
                
                let key = if i == 10 {
                    format!("{}+{}", modifier, 0)
                } else {
                    format!("{}+{}", modifier, i)
                };

                let binding: KeybindingType = engine.eval_expression(&format!("{}({})", binding_name, i))?;

                add_keybinding(engine, scope, key, binding);
            }

            Ok(().into())
        },
    )?;

    Ok(())
}
