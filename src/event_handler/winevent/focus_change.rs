use crate::{system::NativeWindow, AppState};

pub fn handle(
    state: &mut AppState,
    window: NativeWindow,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some((g, _)) = state.find_window(window.id) {
        g.focus_stack.clear();
        g.focused_window_id = Some(window.id);
    }

    Ok(())
}
