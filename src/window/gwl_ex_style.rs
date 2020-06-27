use bitflags::bitflags;
use winapi::um::winuser::WS_EX_ACCEPTFILES;
use winapi::um::winuser::WS_EX_APPWINDOW;
use winapi::um::winuser::WS_EX_CLIENTEDGE;
use winapi::um::winuser::WS_EX_COMPOSITED;
use winapi::um::winuser::WS_EX_CONTEXTHELP;
use winapi::um::winuser::WS_EX_CONTROLPARENT;
use winapi::um::winuser::WS_EX_DLGMODALFRAME;
use winapi::um::winuser::WS_EX_LAYERED;
use winapi::um::winuser::WS_EX_LAYOUTRTL;
use winapi::um::winuser::WS_EX_LEFT;
use winapi::um::winuser::WS_EX_LEFTSCROLLBAR;
use winapi::um::winuser::WS_EX_LTRREADING;
use winapi::um::winuser::WS_EX_MDICHILD;
use winapi::um::winuser::WS_EX_NOACTIVATE;
use winapi::um::winuser::WS_EX_NOINHERITLAYOUT;
use winapi::um::winuser::WS_EX_NOPARENTNOTIFY;
use winapi::um::winuser::WS_EX_NOREDIRECTIONBITMAP;
use winapi::um::winuser::WS_EX_OVERLAPPEDWINDOW;
use winapi::um::winuser::WS_EX_PALETTEWINDOW;
use winapi::um::winuser::WS_EX_RIGHT;
use winapi::um::winuser::WS_EX_RIGHTSCROLLBAR;
use winapi::um::winuser::WS_EX_RTLREADING;
use winapi::um::winuser::WS_EX_STATICEDGE;
use winapi::um::winuser::WS_EX_TOOLWINDOW;
use winapi::um::winuser::WS_EX_TOPMOST;
use winapi::um::winuser::WS_EX_TRANSPARENT;
use winapi::um::winuser::WS_EX_WINDOWEDGE;

bitflags! {
    #[derive(Default)]
    pub struct GwlExStyle: i32 {
        const ACCEPTFILES = WS_EX_ACCEPTFILES as i32;
        const APPWINDOW = WS_EX_APPWINDOW as i32;
        const CLIENTEDGE = WS_EX_CLIENTEDGE as i32;
        const COMPOSITED = WS_EX_COMPOSITED as i32;
        const CONTEXTHELP = WS_EX_CONTEXTHELP as i32;
        const CONTROLPARENT = WS_EX_CONTROLPARENT as i32;
        const DLGMODALFRAME = WS_EX_DLGMODALFRAME as i32;
        const LAYERED = WS_EX_LAYERED as i32;
        const LAYOUTRTL = WS_EX_LAYOUTRTL as i32;
        const LEFT = WS_EX_LEFT as i32;
        const LEFTSCROLLBAR = WS_EX_LEFTSCROLLBAR as i32;
        const LTRREADING = WS_EX_LTRREADING as i32;
        const MDICHILD = WS_EX_MDICHILD as i32;
        const NOACTIVATE = WS_EX_NOACTIVATE as i32;
        const NOINHERITLAYOUT = WS_EX_NOINHERITLAYOUT as i32;
        const NOPARENTNOTIFY = WS_EX_NOPARENTNOTIFY as i32;
        const NOREDIRECTIONBITMAP = WS_EX_NOREDIRECTIONBITMAP as i32;
        const OVERLAPPEDWINDOW = WS_EX_OVERLAPPEDWINDOW as i32;
        const PALETTEWINDOW = WS_EX_PALETTEWINDOW as i32;
        const RIGHT = WS_EX_RIGHT as i32;
        const RIGHTSCROLLBAR = WS_EX_RIGHTSCROLLBAR as i32;
        const RTLREADING = WS_EX_RTLREADING as i32;
        const STATICEDGE = WS_EX_STATICEDGE as i32;
        const TOOLWINDOW = WS_EX_TOOLWINDOW as i32;
        const TOPMOST = WS_EX_TOPMOST as i32;
        const TRANSPARENT = WS_EX_TRANSPARENT as i32;
        const WINDOWEDGE = WS_EX_WINDOWEDGE as i32;
    }
}
