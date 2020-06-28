use crate::change_workspace;
use crate::util;
use crate::window::gwl_ex_style::GwlExStyle;
use crate::window::gwl_style::GwlStyle;
use crate::window::Window;
use crate::CONFIG;
use crate::GRIDS;
use crate::WORKSPACE_ID;
use log::debug;
use winapi::shared::windef::HWND;

pub fn handle(hwnd: HWND, ignore_window_style: bool) -> Result<(), Box<dyn std::error::Error>> {
    let title = util::get_title_of_window(hwnd);

    if title.is_err() {
        return Ok(());
    }

    let mut window = Window {
        id: hwnd as i32,
        title: title.unwrap(),
        ..Window::default()
    };
    window.style = window.get_style().unwrap_or_default();
    window.original_style = window.style;
    window.exstyle = window.get_ex_style().unwrap_or_default();
    let parent = window.get_parent_window();

    let correct_style = ignore_window_style
        || (window.original_style.contains(GwlStyle::CAPTION)
            && !window.exstyle.contains(GwlExStyle::DLGMODALFRAME));

    for rule in CONFIG.lock().unwrap().rules.clone() {
        if rule.pattern.is_match(&window.title) {
            debug!("Rule({:?}) matched!", rule.pattern);
            window.rule = Some(rule.clone());
            break;
        }
    }

    let rule = window.rule.clone().unwrap_or_default();
    let should_manage = rule.manage && parent.is_err() && correct_style;

    if should_manage {
        debug!("Managing window");
        let mut workspace_id = *WORKSPACE_ID.lock().unwrap();

        if rule.workspace != -1 {
            workspace_id = rule.workspace;
            change_workspace(workspace_id)?;
        }

        if CONFIG.lock().unwrap().remove_title_bar {
            window.remove_title_bar();
            window.update_style();
        }

        let mut grids = GRIDS.lock().unwrap();
        let grid = grids.iter_mut().find(|g| g.id == workspace_id).unwrap();

        window.original_rect = window.get_rect()?;

        grid.split(window);

        grid.draw_grid();
    }

    Ok(())
}
