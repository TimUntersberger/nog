use super::{functions, syntax};
use crate::config::Config;
use rhai::Engine;

pub fn parse_config() -> Result<Config, Box<dyn std::error::Error>> {
    let mut engine = Engine::new();
    let mut config = Config::default();

    functions::init(&mut engine)?;
    syntax::init(&mut config, &mut engine)?;

    let mut config_path = std::env::current_dir()?;

    config_path.push("config.rhai");

    engine.consume_file(config_path)?;

    Ok(config)
}
