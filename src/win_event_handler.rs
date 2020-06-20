use crate::util;
use crate::window::gwl_ex_style::GwlExStyle;
use crate::window::gwl_style::GwlStyle;
use crate::window::Window;
use crate::CONFIG;
use crate::GRIDS;
use crate::WORKSPACE_ID;
use log::debug;
use num_traits::FromPrimitive;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::LONG;
use winapi::shared::windef::HWINEVENTHOOK;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::winuser::SetWinEventHook;
use winapi::um::winuser::UnhookWinEvent;
use winapi::um::winuser::EVENT_MAX;
use winapi::um::winuser::EVENT_MIN;
use winapi::um::winuser::EVENT_OBJECT_CREATE;
use winapi::um::winuser::EVENT_OBJECT_DESTROY;
use winapi::um::winuser::EVENT_OBJECT_SHOW;
use winapi::um::winuser::OBJID_WINDOW;

static HANDLED_EVENTS: [u32; 2] = [EVENT_OBJECT_SHOW, EVENT_OBJECT_DESTROY];
static OS_WINDOWS: [&str; 25] = [
    "Task Switching",
    "OLEChannelWnd",
    ".NET-BroadcastEventWindow",
    "Default IME",
    "MSCTFIME UI",
    "OleMainThreadWndName",
    "CicMarshalWnd",
    "Chrome Legacy Window",
    "Windows Shell Experience Host",
    "Shell Preview Extension Host",
    "Namespace Tree Control",
    "ShellView",
    "Pop-upHost",
    "PopupHost",
    "DesktopWindowXamlSource",
    "Running applications",
    "MSCTFTIME UI",
    "XCP",
    "DIEmWin",
    "CSpNotify Notify Window",
    "FolderView",
    "Address: Quick access",
    "Hidden Window",
    "Ribb",
    "wwm_app_bar",
];

static mut HOOK: Option<HWINEVENTHOOK> = None;

fn is_os_window(title: &str) -> bool {
    return OS_WINDOWS.iter().any(|name| title.contains(name));
}

pub fn split_window(window_handle: HWND) -> Result<(), util::WinApiResultError> {
    let window_title = util::get_title_of_window(window_handle)?;
    handle_event_object_show(window_handle, &window_title, true)
}

fn handle_event_object_show(
    window_handle: HWND,
    window_title: &str,
    ignore_window_style: bool,
) -> Result<(), util::WinApiResultError> {
    // gets the GWL_STYLE of the window. GWL_STYLE returns a bitmask that can be used to find out attributes about a window
    let mut window = Window {
        id: window_handle as i32,
        title: window_title.to_string(),
        ..Window::default()
    };
    window.original_style = window.get_style()?;

    let exstyle = window.get_ex_style()?;
    let parent = window.get_parent_window();

    let correct_style = ignore_window_style
        || (window.original_style.contains(GwlStyle::CAPTION)
            && !exstyle.contains(GwlExStyle::DLGMODALFRAME));

    for rule in &CONFIG.rules {
        if rule.pattern.is_match(&window.title) {
            debug!("Rule({:?}) matched!", rule.pattern);
            window.rule = Some(rule.clone());
            break;
        }
    }

    let should_manage = parent.is_err() && correct_style;


    if should_manage {
        debug!("Managing window");

        if CONFIG.remove_title_bar {
            window.remove_title_bar()?;
        }
        let mut grids = GRIDS.lock().unwrap();
        let grid = grids
            .iter_mut()
            .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
            .unwrap();
        window.original_rect = window.get_rect()?;

        grid.split(window);

        grid.print_grid();
    }

    Ok(())
}
fn handle_event_object_destroy(window_handle: HWND) {
    let mut grids = GRIDS.lock().unwrap();
    let grid = grids
        .iter_mut()
        .find(|g| g.id == *WORKSPACE_ID.lock().unwrap())
        .unwrap();
    grid.close_tile_by_window_id(window_handle as i32);
    grid.print_grid();
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
enum WinEvent {
    EventObjectCreate = EVENT_OBJECT_CREATE as isize,
    EventObjectDestroy = EVENT_OBJECT_DESTROY as isize,
    EventObjectShow = EVENT_OBJECT_SHOW as isize,
    Unknown,
}

unsafe extern "system" fn handler(
    _: HWINEVENTHOOK,
    event: DWORD,
    window_handle: HWND,
    object_type: LONG,
    _: LONG,
    _: DWORD,
    _: DWORD,
) {
    if HANDLED_EVENTS.contains(&event) {
        if object_type != OBJID_WINDOW {
            return;
        }

        let res = util::get_title_of_window(window_handle);

        if res.is_err() {
            return;
        }

        let window_title = res.unwrap();

        if is_os_window(&window_title) {
            return;
        }

        debug!(
            "{:?}({}): '{}' | {}",
            WinEvent::from_u32(event).unwrap_or(WinEvent::Unknown),
            event,
            window_title,
            window_handle as i32
        );

        let res = match event {
            EVENT_OBJECT_SHOW => handle_event_object_show(window_handle, &window_title, false),
            EVENT_OBJECT_DESTROY => Ok(handle_event_object_destroy(window_handle)),
            _ => Ok(()),
        };

        if let Err(error) = res {
            println!("{}", error);
        }
    }
}

pub fn register() -> Result<(), util::WinApiResultError> {
    unsafe {
        let hook = util::winapi_ptr_to_result(SetWinEventHook(
            EVENT_MIN,
            EVENT_MAX,
            std::ptr::null_mut(),
            Some(handler),
            0,
            0,
            0,
        ))?;

        HOOK = Some(hook);
    }

    Ok(())
}

pub fn unregister() -> Result<(), util::WinApiResultError> {
    unsafe {
        if HOOK.is_some() {
            util::winapi_err_to_result(UnhookWinEvent(HOOK.unwrap()))?;
        }
    }

    Ok(())
}
