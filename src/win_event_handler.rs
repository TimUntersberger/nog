use crate::util;
use crate::window::Window;
use crate::CONFIG;
use crate::GRID;
use winapi::shared::minwindef::BOOL;
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
use winapi::um::winuser::WS_CAPTION;

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
        name: window_title.to_string(),
        original_style: 0,
        original_rect: RECT::default(),
    };
    window.original_style = window.get_style()?;
    let parent = window.get_parent_window();
    // checks whether the window has a titlebar
    // if the window doesn't have a titlebar, it usually means that we shouldn't manage the window (because it's a tooltip or something like that)
    let should_manage = parent.is_err()
        && (ignore_window_style
            || (window.original_style & WS_CAPTION as i32) == WS_CAPTION as i32);
    println!("should manage is {}", should_manage);

    if should_manage {
        if CONFIG.remove_title_bar {
            window.remove_title_bar()?;
        }

        if let Ok(mut grid) = GRID.lock() {
            window.original_rect = window.get_rect()?;

            grid.split(window);

            grid.print_grid();
        }
    }

    Ok(())
}
fn handle_event_object_destroy(window_handle: HWND) {
    if let Ok(mut grid) = GRID.lock() {
        grid.close_tile_by_window_id(window_handle as i32);
        grid.print_grid();
    }
}
unsafe extern "system" fn handler(
    _: HWINEVENTHOOK,
    event: DWORD,
    window_handle: HWND,
    object_type: LONG,
    child: LONG,
    _: DWORD,
    _: DWORD,
) {
    if HANDLED_EVENTS.contains(&event) {
        if object_type != OBJID_WINDOW {
            return;
        }

        let res = util::get_title_of_window(window_handle);

        if res.is_err() {
            return
        }

        let window_title = res.unwrap();

        if is_os_window(&window_title) {
            return;
        }

        println!(
            "[{}] {}({} | {}, {}): '{}' {}",
            chrono::offset::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            match event {
                EVENT_OBJECT_CREATE => "EVENT_OBJECT_CREATE",
                EVENT_OBJECT_DESTROY => "EVENT_OBJECT_DESTROY",
                EVENT_OBJECT_SHOW => "EVENT_OBJECT_SHOW",
                _ => "UNKNOWN",
            },
            event,
            object_type,
            child,
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

pub fn unregister() -> util::WinApiResult<BOOL> {
    unsafe {
        return util::winapi_err_to_result(UnhookWinEvent(HOOK.unwrap()));
    }
}
