use super::{key::Key, keybinding_type::KeybindingType, key_press::KeyPress};

#[derive(Debug, Clone)]
pub struct Keybinding {
    pub typ: KeybindingType,
    pub key: Key,
    pub shift: bool,
    pub alt: bool,
    pub control: bool,
    pub registered: bool,
}

impl From<KeyPress> for Keybinding {
    fn from(kp: KeyPress) -> Self {
        Self {
            shift: kp.shift,
            alt: kp.alt,
            control: kp.control,
            key: kp.key,
            registered: false,
            typ: KeybindingType::Quit
        }
    }
}

impl PartialEq<KeyPress> for Keybinding {
    fn eq(&self, other: &KeyPress) -> bool {
        self.control == other.control && self.alt == other.alt && self.shift == other.shift && self.key == other.key
    }
}