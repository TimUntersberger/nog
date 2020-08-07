use super::{draw_workspace, WINDOWS};
use crate::{tile_grid::TileGrid, workspace::is_visible_workspace, CONFIG, GRIDS, WORKSPACE_ID};
use log::debug;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::wingdi::DeleteObject;
use winapi::um::winuser::{FillRect, GetClientRect, GetDC};

pub fn draw(hwnd: HWND) {
    let grids = GRIDS.lock().unwrap();

    let monitor = *WINDOWS
        .lock()
        .unwrap()
        .iter()
        .find(|(_, v)| **v == hwnd as i32)
        .map(|(m, _)| m)
        .expect("Couldn't find monitor for appbar");

    debug!("On monitor {}", monitor as i32);

    let workspaces: Vec<&TileGrid> = grids
        .iter()
        .filter(|g| {
            (!g.tiles.is_empty() || is_visible_workspace(g.id)) && g.display.hmonitor == monitor
        })
        .collect();

    //erase last workspace
    erase_workspace(hwnd, (workspaces.len()) as i32);

    for (i, workspace) in workspaces.iter().enumerate() {
        draw_workspace::draw(
            hwnd,
            i as i32,
            workspace.id,
            *WORKSPACE_ID.lock().unwrap() == workspace.id,
        )
        .expect("Failed to draw workspace");
    }
}

fn erase_workspace(hwnd: HWND, id: i32) {
    unsafe {
        let mut rect = RECT::default();
        let app_bar_height = CONFIG.lock().unwrap().app_bar_height;
        let app_bar_bg = CONFIG.lock().unwrap().app_bar_bg;
        let brush = CreateSolidBrush(app_bar_bg as u32);

        let hdc = GetDC(hwnd);
        GetClientRect(hwnd, &mut rect);

        rect.left += app_bar_height * id;
        rect.right = rect.left + app_bar_height;

        FillRect(hdc, &rect, brush);

        DeleteObject(brush as *mut std::ffi::c_void);
    }
}
