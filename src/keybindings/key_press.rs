use std::{fmt::Debug, collections::VecDeque, str::FromStr};
use super::key::Key;
use log::error;

#[derive(Copy, Clone)]
pub struct KeyPress {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub key: Key,
}

impl Debug for KeyPress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut modifiers: VecDeque<String> = vec![format!("{}", self.key)].into();

        if self.alt {
            modifiers.push_front(String::from("Alt"));
        }

        if self.shift {
            modifiers.push_front(String::from("Shift"));
        }

        if self.control {
            modifiers.push_front(String::from("Control"));
        }

        f.write_str(&modifiers.into_iter().collect::<Vec<String>>().join("+"))
    }
}

impl PartialEq for KeyPress {
    fn eq(&self, other: &Self) -> bool {
        self.control == other.control && self.alt == other.alt && self.shift == other.shift && self.key == other.key
    }
}

impl Default for KeyPress {
    fn default() -> Self {
        Self {
            shift: false,
            alt: false,
            control: false,
            key: Key::A
        }
    }
}

impl FromStr for KeyPress {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut key_press = KeyPress::default();
        let tokens: Vec<&str> = s.split("+").collect();

        for token in tokens.iter().take(tokens.len() - 1) {
            match *token {
                "Alt" => key_press.alt = true,
                "Control" => key_press.control = true,
                "Shift" => key_press.shift = true,
                _ => {
                    error!("Unknown modifier {}", *token);
                }
            }
        }

        key_press.key = Key::from_str(tokens.last().unwrap())?;

        Ok(key_press)
    }
}