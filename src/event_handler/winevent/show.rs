use crate::{
    util, window::gwl_ex_style::GwlExStyle, window::gwl_style::GwlStyle, window::Window,
    workspace::change_workspace, ADDITIONAL_RULES, CONFIG, GRIDS, WORKSPACE_ID,
};
use log::debug;
use winapi::shared::windef::HWND;

pub fn handle(hwnd: HWND, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let title = util::get_title_of_window(hwnd);
    let min_width = CONFIG.lock().min_width;
    let min_height = CONFIG.lock().min_height;

    if title.is_err() {
        return Ok(());
    }

    let mut window = Window {
        id: hwnd as i32,
        title: title.unwrap(),
        ..Window::default()
    };

    let rect = window.get_client_rect();

    if !force && (rect.right - rect.left < min_width || rect.bottom - rect.top < min_height) {
        return Ok(());
    }

    window.original_style = window.get_style().unwrap_or_default();
    if window.original_style.contains(GwlStyle::MAXIMIZE) {
        window.restore();
        window.maximized = true;
        window.original_style.remove(GwlStyle::MAXIMIZE);
    }
    window.style = window.original_style;
    window.exstyle = window.get_ex_style().unwrap_or_default();

    let parent = window.get_parent_window();

    let correct_style = force
        || (window.original_style.contains(GwlStyle::CAPTION)
            && !window.exstyle.contains(GwlExStyle::DLGMODALFRAME));

    let additional_rules = ADDITIONAL_RULES.lock();

    for rule in CONFIG
        .lock()
        .rules
        .iter()
        .chain(additional_rules.iter())
    {
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
    let should_manage = force || (rule.manage && parent.is_err() && correct_style);

    if should_manage {
        debug!("Managing window");
        let mut workspace_id = *WORKSPACE_ID.lock();

        if rule.workspace_id != -1 {
            workspace_id = rule.workspace_id;
            change_workspace(workspace_id, false)?;
        }

        if CONFIG.lock().remove_title_bar {
            window.remove_title_bar();
            window.update_style();
        }

        let mut grids = GRIDS.lock();
        let grid = grids.iter_mut().find(|g| g.id == workspace_id).unwrap();

        window.original_rect = window.get_rect()?;

        grid.split(window);

        grid.draw_grid();
    }

    Ok(())
}
