use super::{key::Key, keybinding_type::KeybindingType, modifier::Modifier};
use std::{fmt::Debug, str::FromStr};

#[derive(Clone)]
pub struct Keybinding {
    pub typ: KeybindingType,
    pub key: Key,
    pub modifier: Modifier,
}

impl FromStr for Keybinding {
    type Err = Box<dyn std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key_combo_parts = s.split('+').collect::<Vec<&str>>();
        let modifier_count = key_combo_parts.len() - 1;

        let modifier = key_combo_parts
            .iter()
            .take(modifier_count)
            .map(|x| match *x {
                "Alt" => Modifier::ALT,
                "Control" => Modifier::CONTROL,
                "Shift" => Modifier::SHIFT,
                _ => Modifier::default(),
            })
            .fold(Modifier::default(), |mut sum, crr| {
                sum.insert(crr);

                sum
            });

        let key = key_combo_parts
            .iter()
            .last()
            .and_then(|x| Key::from_str(x).ok())
            .ok_or("Invalid key")?;

        Ok(Self {
            typ: KeybindingType::Quit,
            modifier,
            key,
        })
    }
}

impl Debug for Keybinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Keybinding({}+{:?}, {:?})",
            format!("{:?}", self.modifier).replace(" | ", "+"),
            self.key,
            self.typ
        ))
    }
}
