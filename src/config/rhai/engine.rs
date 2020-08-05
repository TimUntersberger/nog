use super::{functions, syntax};
use crate::{keybindings::keybinding::Keybinding, config::Config};
use rhai::{Engine, Scope, Array};

pub fn parse_config() -> Result<Config, Box<dyn std::error::Error>> {
    let mut engine = Engine::new();
    let mut scope = Scope::new();
    let mut config = Config::default();

    scope.set_value("__keybindings", Array::new());

    functions::init(&mut engine)?;
    syntax::init(&mut engine)?;

    let mut config_path = std::env::current_dir()?;

    config_path.push("config.rhai");

    engine.consume_file_with_scope(&mut scope, config_path)?;

    let keybindings: Array = scope.get_value("__keybindings").unwrap();

    for val in keybindings {
        let boxed = val.cast::<Box<Keybinding>>();
        config.keybindings.push(*boxed);
    }

    Ok(config)
}
