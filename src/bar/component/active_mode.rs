use super::{Component, ComponentText};

pub fn create() -> Component {
    Component::new(
        "ActiveMode",
        |ctx| {
            vec![ComponentText::new().with_display_text(
                ctx.state
                    .keybindings_manager
                    .get_mode()
                    .map(|m| format!("{} is active", m))
                    .unwrap_or_default(),
            )]
        },
    )
}
