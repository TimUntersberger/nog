use crate::{
    system::NativeWindow, workspace::change_workspace, ADDITIONAL_RULES, CONFIG, GRIDS,
    WORKSPACE_ID,
};
use log::debug;

pub fn handle(mut window: NativeWindow, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let min_width = CONFIG.lock().min_width;
    let min_height = CONFIG.lock().min_height;

    let rect = window.get_rect()?;

    if !force && (rect.right - rect.left < min_width || rect.bottom - rect.top < min_height) {
        return Ok(());
    }

    window.init()?;

    let parent = window.get_parent_window();

    let additional_rules = ADDITIONAL_RULES.lock();

    for rule in CONFIG.lock().rules.iter().chain(additional_rules.iter()) {
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
        let mut workspace_id = *WORKSPACE_ID.lock();

        if rule.workspace_id != -1 {
            workspace_id = rule.workspace_id;
            change_workspace(workspace_id, false);
        }

        if CONFIG.lock().remove_title_bar {
            window.remove_title_bar()?;
        }

        let mut grids = GRIDS.lock();
        let grid = grids.iter_mut().find(|g| g.id == workspace_id).unwrap();

        grid.split(window);

        grid.draw_grid()?;
    }

    Ok(())
}
