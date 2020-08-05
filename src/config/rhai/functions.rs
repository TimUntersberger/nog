use crate::{direction::Direction, keybindings::keybinding_type::KeybindingType};
use rhai::{Engine, RegisterFn};
use std::str::FromStr;

pub fn init(engine: &mut Engine) -> Result<(), Box<dyn std::error::Error>> {
    engine.register_fn("launch", |program: String| -> KeybindingType {
        KeybindingType::Launch(program)
    });
    engine.register_fn("focus", |direction: String| -> KeybindingType {
        KeybindingType::Focus(Direction::from_str(&direction).unwrap())
    });

    Ok(())
}
