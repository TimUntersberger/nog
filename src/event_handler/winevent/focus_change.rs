use crate::{system::NativeWindow, system::SystemResult, AppState};

pub fn handle(state: &mut AppState, window: NativeWindow) -> SystemResult {
    if let Some((g, _)) = state.find_window(window.id) {
        g.focus_stack.clear();
        g.focused_window_id = Some(window.id);
        state.workspace_id = g.id;
    }

    Ok(())
}
