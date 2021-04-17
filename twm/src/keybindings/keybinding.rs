use super::{key::Key, modifier::Modifier};
use std::{fmt::Debug, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeybindingKind {
    /// always active
    Global,
    /// active when in work mode
    Work,
    /// default
    Normal,
}

impl Default for KeybindingKind {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Clone, PartialEq)]
pub struct Keybinding {
    pub kind: KeybindingKind,
    /// This is the id of the callback in the global callbacks store
    pub callback_id: usize,
    pub mode: Option<String>,
    pub key: Key,
    pub modifier: Modifier,
}

impl Keybinding {
    pub fn get_id(&self) -> i32 {
        (self.key as u32 + self.modifier.bits() * 1000) as i32
    }

    pub fn is_global(&self) -> bool {
        self.kind == KeybindingKind::Global
    }

    pub fn is_work(&self) -> bool {
        self.kind == KeybindingKind::Work
    }

    pub fn is_normal(&self) -> bool {
        self.kind == KeybindingKind::Normal
    }
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

        let raw_key = key_combo_parts.iter().last().unwrap();
        let key = Key::from_str(raw_key)
            .ok()
            .ok_or(format!("Invalid key {}", raw_key))?;

        Ok(Self {
            kind: KeybindingKind::default(),
            callback_id: 0,
            mode: None,
            modifier,
            key,
        })
    }
}

impl Debug for Keybinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let modifier_str = format!("{:?}", self.modifier).replace(" | ", "+");
        if modifier_str == "(empty)" {
            f.write_str(&format!(
                "Keybinding({:?}, {}, {:?}, {}, {:?})",
                self.key,
                self.callback_id,
                self.kind,
                self.get_id(),
                self.mode
            ))
        } else {
            f.write_str(&format!(
                "Keybinding({}+{:?}, {}, {:?}, {}, {:?})",
                modifier_str,
                self.key,
                self.callback_id,
                self.kind,
                self.get_id(),
                self.mode
            ))
        }
    }
}
