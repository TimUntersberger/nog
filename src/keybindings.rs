mod hook;
pub mod key;
pub mod key_press;
pub mod keybinding;
pub mod keybinding_type;

pub fn register() {
    hook::register();
}

pub fn unregister() {
    hook::unregister();
}
