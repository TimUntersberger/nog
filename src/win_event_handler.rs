use crate::GRID;
use crate::CONFIG;
use crate::window::Window;
use crate::util::get_title_of_window;
use winapi::um::winuser::EVENT_OBJECT_CREATE;
use winapi::um::winuser::EVENT_OBJECT_SHOW;
use winapi::um::winuser::EVENT_OBJECT_DESTROY;
use winapi::um::winuser::GetParent;
use winapi::um::winuser::GetWindowLongA;
use winapi::um::winuser::SetWindowLongA;
use winapi::um::winuser::GetWindowRect;
use winapi::um::winuser::GWL_STYLE;
use winapi::um::winuser::WS_CAPTION;
use winapi::um::winuser::WS_THICKFRAME;
use winapi::um::winuser::WS_BORDER;
use winapi::shared::windef::HWND;
use winapi::shared::windef::HWINEVENTHOOK;
use winapi::shared::windef::RECT;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::LONG;
use winapi::um::winuser::OBJID_WINDOW;
use winapi::um::winuser::SetWinEventHook;
use winapi::um::winuser::UnhookWinEvent;
use winapi::um::winuser::EVENT_MIN;
use winapi::um::winuser::EVENT_MAX;

static HANDLED_EVENTS: [u32; 2] = [EVENT_OBJECT_SHOW, EVENT_OBJECT_DESTROY];
static OS_WINDOWS: [&str; 24] = [
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
    "Ribb"
];

static mut hook: Option<HWINEVENTHOOK> = None;

fn is_os_window(title: &str) -> bool {
    return OS_WINDOWS.iter().any(|name| title.contains(name));
}

pub unsafe fn split_window(window_handle: HWND) {
    if let Some(window_title) = get_title_of_window(window_handle) {
        handle_event_object_show(window_handle, &window_title, true);
    }
}

unsafe fn handle_event_object_show(window_handle: HWND, window_title: &str, ignore_window_style: bool){
    // gets the GWL_STYLE of the window. GWL_STYLE returns a bitmask that can be used to find out attributes about a window
    let window_style = GetWindowLongA(window_handle, GWL_STYLE);
    let parent = GetParent(window_handle) as i32;
    // checks whether the window has a titlebar
    // if the window doesn't have a titlebar, it usually means that we shouldn't manage the window (because it's a tooltip or something like that)
    let should_manage = parent == 0 && (ignore_window_style || (window_style & WS_CAPTION as i32) == WS_CAPTION as i32);
    println!("should manage is {}", should_manage);

    if should_manage {
        if CONFIG.remove_title_bar {
            SetWindowLongA(window_handle, GWL_STYLE, 
                window_style & 
                !WS_CAPTION as i32 & 
                !WS_THICKFRAME as i32 |
                WS_BORDER as i32
            );
        }

        if let Ok(mut grid) = GRID.lock() {
            let mut rect: RECT = RECT { bottom: 0, left: 0, right: 0, top: 0};

            GetWindowRect(window_handle, &mut rect);

            grid.split(Window {
                id: window_handle as i32,
                name: window_title.to_string(),
                original_style: window_style,
                original_rect: rect
            });

            grid.print_grid();
        
        }
    }
}
unsafe fn handle_event_object_destroy(window_handle: HWND){
    if let Ok(mut grid) = GRID.lock() {
        grid.close_tile_by_window_id(window_handle as i32);
        grid.print_grid();
    }
}
unsafe extern "system" fn handler(_: HWINEVENTHOOK, event: DWORD, window_handle: HWND, object_type: LONG, child: LONG, _: DWORD, _: DWORD) {
    if HANDLED_EVENTS.contains(&event) {
        if object_type != OBJID_WINDOW {
            return;
        }  
        if let Some(window_title) = get_title_of_window(window_handle) {
            if is_os_window(&window_title)
            {
                return;
            }

            println!("[{}] {}({} | {}, {}): '{}' {}",  chrono::offset::Utc::now().format("%Y-%m-%d %H:%M:%S"), match event {
                EVENT_OBJECT_CREATE => "EVENT_OBJECT_CREATE",
                EVENT_OBJECT_DESTROY => "EVENT_OBJECT_DESTROY",
                EVENT_OBJECT_SHOW => "EVENT_OBJECT_SHOW",
                _ => "UNKNOWN"
            }, event, object_type, child, window_title, window_handle as i32);

            match event {
                EVENT_OBJECT_SHOW => handle_event_object_show(window_handle, &window_title, false),
                EVENT_OBJECT_DESTROY => handle_event_object_destroy(window_handle),
                _ => {}
            }
        }
    }
}

pub unsafe fn register(){
    hook = Some(SetWinEventHook(
        EVENT_MIN, 
        EVENT_MAX, 
        std::ptr::null_mut(),
        Some(handler),
        0,
        0,
        0
    ));
}

pub unsafe fn unregister(){
    UnhookWinEvent(hook.unwrap());
}
