use crate::{system::NativeWindow, system::SystemResult, AppState};
use log::{debug, error};

pub fn handle(state: &mut AppState, mut window: NativeWindow, force: bool) -> SystemResult {
    let min_width = state.config.min_width;
    let min_height = state.config.min_height;

    let config = state.config.clone();
    let rect = fail!(window
        .get_rect()
        .map_err(|_| "Failed to get rectangle of new window"));

    if !force && (rect.right - rect.left < min_width || rect.bottom - rect.top < min_height) {
        return Ok(());
    }

    window.init()?;

    let parent = window.get_parent_window();

    for rule in config.rules.iter().chain(state.additonal_rules.iter()) {
        // checks for path
        let process_name = if rule.pattern.to_string().contains('\\') {
            window.get_process_path()
        } else {
            window.get_process_name()
        };

        let window_name = window.title.clone();

        if rule.pattern.is_match(&process_name) || rule.pattern.is_match(&window_name) {
            debug!("Rule({:?}) matched!", rule.pattern);
            window.rule = Some(rule.clone());
            break;
        }
    }

    let rule = window.rule.clone().unwrap_or_default();
    let should_manage = force || (rule.manage && parent.is_err() && window.should_manage());

    if should_manage {
        debug!("Managing window");
        if rule.workspace_id != -1 {
            state.change_workspace(rule.workspace_id, false);
        }

        if config.remove_title_bar {
            window.remove_title_bar(config.use_border)?;
        }

        let display = state.get_current_display_mut();
        if let Some(grid) = display.get_focused_grid_mut() {
            grid.split(window);
        }
        display.refresh_grid(&config)?;
    }

    Ok(())
}
