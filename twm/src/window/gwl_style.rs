use bitflags::bitflags;
use winapi::um::winuser::WS_BORDER;
use winapi::um::winuser::WS_CAPTION;
use winapi::um::winuser::WS_CHILD;
use winapi::um::winuser::WS_CHILDWINDOW;
use winapi::um::winuser::WS_CLIPCHILDREN;
use winapi::um::winuser::WS_CLIPSIBLINGS;
use winapi::um::winuser::WS_DISABLED;
use winapi::um::winuser::WS_DLGFRAME;
use winapi::um::winuser::WS_GROUP;
use winapi::um::winuser::WS_HSCROLL;
use winapi::um::winuser::WS_ICONIC;
use winapi::um::winuser::WS_MAXIMIZE;
use winapi::um::winuser::WS_MAXIMIZEBOX;
use winapi::um::winuser::WS_MINIMIZE;
use winapi::um::winuser::WS_MINIMIZEBOX;
use winapi::um::winuser::WS_OVERLAPPED;
use winapi::um::winuser::WS_OVERLAPPEDWINDOW;
use winapi::um::winuser::WS_POPUP;
use winapi::um::winuser::WS_POPUPWINDOW;
use winapi::um::winuser::WS_SIZEBOX;
use winapi::um::winuser::WS_SYSMENU;
use winapi::um::winuser::WS_TABSTOP;
use winapi::um::winuser::WS_THICKFRAME;
use winapi::um::winuser::WS_TILED;
use winapi::um::winuser::WS_TILEDWINDOW;
use winapi::um::winuser::WS_VISIBLE;
use winapi::um::winuser::WS_VSCROLL;

bitflags! {
    #[derive(Default)]
    pub struct GwlStyle: i32 {
        const BORDER = WS_BORDER as i32;
        const CAPTION = WS_CAPTION as i32;
        const CHILD = WS_CHILD as i32;
        const CHILDWINDOW = WS_CHILDWINDOW as i32;
        const CLIPCHILDREN = WS_CLIPCHILDREN as i32;
        const CLIPSIBLINGS = WS_CLIPSIBLINGS as i32;
        const DISABLED = WS_DISABLED as i32;
        const DLGFRAME = WS_DLGFRAME as i32;
        const GROUP = WS_GROUP as i32;
        const HSCROLL = WS_HSCROLL as i32;
        const ICONIC = WS_ICONIC as i32;
        const MAXIMIZE = WS_MAXIMIZE as i32;
        const MAXIMIZEBOX = WS_MAXIMIZEBOX as i32;
        const MINIMIZE = WS_MINIMIZE as i32;
        const MINIMIZEBOX = WS_MINIMIZEBOX as i32;
        const OVERLAPPED = WS_OVERLAPPED as i32;
        const OVERLAPPEDWINDOW = WS_OVERLAPPEDWINDOW as i32;
        const POPUP = WS_POPUP as i32;
        const POPUPWINDOW = WS_POPUPWINDOW as i32;
        const SIZEBOX = WS_SIZEBOX as i32;
        const SYSMENU = WS_SYSMENU as i32;
        const TABSTOP = WS_TABSTOP as i32;
        const THICKFRAME = WS_THICKFRAME as i32;
        const TILED = WS_TILED as i32;
        const TILEDWINDOW = WS_TILEDWINDOW as i32;
        const VISIBLE = WS_VISIBLE as i32;
        const VSCROLL = WS_VSCROLL as i32;
    }
}
