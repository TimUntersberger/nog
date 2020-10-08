use super::{Component, ComponentText};
use std::sync::Arc;

pub fn create() -> Component {
    Component::new(
        "ActiveMode",
        Arc::new(|ctx| {
            vec![ComponentText::Basic(
                ctx.state
                    .keybindings_manager
                    .get_mode()
                    .map(|m| format!("{} is active", m))
                    .unwrap_or_default(),
            )]
        }),
    )
}
