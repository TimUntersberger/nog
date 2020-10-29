use log::error;
use parking_lot::Mutex;
use std::{
    ffi::c_void, ffi::CString, sync::atomic::AtomicBool, sync::atomic::Ordering,
    sync::mpsc::channel, sync::mpsc::Receiver, sync::mpsc::Sender, sync::Arc, thread,
    time::Duration,
};
use thread::JoinHandle;
use winapi::um::wingdi::SelectObject;
use winapi::um::wingdi::LOGFONTA;
use winapi::um::wingdi::{GetBValue, GetGValue, GetRValue, RGB};
use winapi::um::{wingdi::CreateFontIndirectA, winuser::IDC_HAND, winuser::WM_MOUSEMOVE};
use winapi::um::{wingdi::DeleteObject, winuser::DT_SINGLELINE, winuser::DT_VCENTER};
use winapi::{
    shared::minwindef::LPARAM, shared::minwindef::LRESULT, shared::minwindef::UINT,
    shared::minwindef::WPARAM, shared::windef::HDC, shared::windef::HWND, shared::windef::POINT,
    shared::windef::RECT, um::wingdi::CreateSolidBrush, um::wingdi::SetBkColor,
    um::wingdi::SetTextColor, um::winuser::BeginPaint, um::winuser::CreateWindowExA,
    um::winuser::DefWindowProcA, um::winuser::DrawTextW, um::winuser::EndPaint,
    um::winuser::FillRect, um::winuser::GetCursorPos, um::winuser::GetDC, um::winuser::LoadCursorA,
    um::winuser::PostMessageA, um::winuser::RegisterClassA, um::winuser::ReleaseDC,
    um::winuser::SetCursor, um::winuser::UnregisterClassA, um::winuser::DT_CALCRECT,
    um::winuser::IDC_ARROW, um::winuser::PAINTSTRUCT, um::winuser::WM_APP, um::winuser::WM_CLOSE,
    um::winuser::WM_CREATE, um::winuser::WM_LBUTTONDOWN, um::winuser::WM_PAINT,
    um::winuser::WM_SETCURSOR, um::winuser::WNDCLASSA, um::winuser::WS_BORDER,
    um::winuser::WS_EX_NOACTIVATE, um::winuser::WS_EX_TOPMOST, um::winuser::WS_OVERLAPPEDWINDOW,
    um::winuser::WS_POPUPWINDOW,
};

use crate::{
    display::Display, message_loop, system::NativeWindow, system::Rectangle, system::SystemResult,
    system::WindowId, util, AppState,
};

pub mod gwl_ex_style;
pub mod gwl_style;

const WM_IDENT: u32 = WM_APP + 80;

#[derive(Debug, Copy, Clone)]
pub struct WindowMsg {
    pub hwnd: HWND,
    pub code: u32,
    pub params: (usize, isize),
}

unsafe extern "system" fn window_cb(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_SETCURSOR {
        return 1;
    } else if msg != WM_IDENT {
        let payload = WindowMsg {
            code: msg,
            hwnd,
            params: (w_param, l_param),
        };

        let ptr = Box::into_raw(Box::new(payload));

        PostMessageA(hwnd, WM_IDENT, ptr as usize, 0);
    }

    DefWindowProcA(hwnd, msg, w_param, l_param)
}

fn convert_color_to_winapi(color: u32) -> u32 {
    RGB(GetRValue(color), GetGValue(color), GetBValue(color))
}

#[derive(Debug, Clone)]
pub struct Api {
    pub hdc: i32,
    pub background_color: i32,
    pub window: NativeWindow,
    pub display: Display,
}

impl Api {
    pub fn set_clickable_cursor(&self) {
        unsafe {
            SetCursor(LoadCursorA(std::ptr::null_mut(), IDC_HAND as *const i8));
        }
    }
    pub fn set_default_cursor(&self) {
        unsafe {
            SetCursor(LoadCursorA(std::ptr::null_mut(), IDC_ARROW as *const i8));
        }
    }
    pub fn set_text_color(&self, color: i32) {
        unsafe {
            SetTextColor(self.hdc as HDC, convert_color_to_winapi(color as u32));
        }
    }
    pub fn set_background_color(&self, color: i32) {
        unsafe {
            SetBkColor(self.hdc as HDC, convert_color_to_winapi(color as u32));
        }
    }
    pub fn reset_background_color(&self) {
        self.set_background_color(self.background_color)
    }
    pub fn fill_rect(&self, x: i32, y: i32, width: i32, height: i32, color: i32) {
        unsafe {
            let brush = CreateSolidBrush(convert_color_to_winapi(color as u32));
            let mut rect = RECT {
                left: x,
                right: x + width,
                top: y,
                bottom: y + height,
            };

            FillRect(self.hdc as HDC, &mut rect, brush);

            DeleteObject(brush as *mut c_void);
        }
    }
    pub fn calculate_text_rect(&self, text: &str) -> Rectangle {
        let c_text = util::to_widestring(&text);
        let mut rect = RECT::default();
        unsafe {
            DrawTextW(self.hdc as HDC, c_text.as_ptr(), -1, &mut rect, DT_CALCRECT);
        }
        rect.into()
    }
    pub fn write_text(&self, text: &str, x: i32, y: i32, vcenter: bool, _hcenter: bool) {
        let c_text = util::to_widestring(&text);
        let mut rect = self.calculate_text_rect(text);

        rect.left += x;
        rect.right += x;
        rect.top += y;
        rect.bottom += y;

        let mut rect = rect.into();
        let mut flags = 0;

        if vcenter {
            flags = DT_VCENTER | DT_SINGLELINE;
        }

        unsafe {
            DrawTextW(self.hdc as HDC, c_text.as_ptr(), -1, &mut rect, flags);
        }
    }
}

#[derive(Debug)]
pub enum WindowEvent<'a> {
    Click {
        display: &'a Display,
        id: WindowId,
        x: i32,
        y: i32,
        state: &'a AppState,
    },
    Create {
        display: &'a Display,
        id: WindowId,
    },
    Close {
        display: &'a Display,
        id: WindowId,
    },
    Draw {
        display: &'a Display,
        id: WindowId,
        state: &'a AppState,
        api: Api,
    },
    MouseMove {
        display: &'a Display,
        id: WindowId,
        api: Api,
        x: i32,
        y: i32,
    },
    Native {
        msg: WindowMsg,
        display: &'a Display,
    },
}

#[derive(Default, Debug)]
struct WindowInner {
    pub native_window: Option<NativeWindow>,
    pub is_popup: bool,
    pub border: bool,
    pub x: i32,
    pub y: i32,
    pub background_color: i32,
    pub height: i32,
    pub width: i32,
    pub title: String,
    pub font: String,
    pub font_size: i32,
}

impl WindowInner {
    pub fn new() -> Self {
        Self {
            title: "nog temp window name".into(),
            border: true,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Window {
    pub id: WindowId,
    pub refresh_rate: i32,
    is_closed: Arc<AtomicBool>,
    inner: Arc<Mutex<WindowInner>>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            id: WindowId::default(),
            refresh_rate: 0,
            is_closed: Arc::new(AtomicBool::new(false)),
            inner: Arc::new(Mutex::new(WindowInner::new())),
        }
    }
    pub fn with_size(self, width: i32, height: i32) -> Self {
        self.inner.lock().height = height;
        self.inner.lock().width = width;
        self
    }
    pub fn with_refresh_rate(mut self, value: i32) -> Self {
        self.refresh_rate = value;
        self
    }
    pub fn with_background_color(self, color: i32) -> Self {
        self.inner.lock().background_color = color;
        self
    }
    pub fn with_font(self, font: &str) -> Self {
        self.inner.lock().font = font.into();
        self
    }
    pub fn with_title(self, title: &str) -> Self {
        self.inner.lock().title = title.into();
        self
    }
    pub fn with_font_size(self, font_size: i32) -> Self {
        self.inner.lock().font_size = font_size;
        self
    }
    pub fn with_pos(self, x: i32, y: i32) -> Self {
        self.inner.lock().x = x;
        self.inner.lock().y = y;
        self
    }
    pub fn with_is_popup(self, val: bool) -> Self {
        self.inner.lock().is_popup = val;
        self
    }
    pub fn with_border(self, val: bool) -> Self {
        self.inner.lock().border = val;
        self
    }
    pub fn get_native_window(&self) -> NativeWindow {
        self.id.into()
    }
    pub fn redraw(&self) -> SystemResult {
        self.get_native_window().redraw()
    }
    pub fn hide(&self) {
        self.get_native_window().hide();
    }
    pub fn show(&self) {
        self.get_native_window().show();
    }
    pub fn close(&self) -> SystemResult {
        self.get_native_window().close()?;

        self.is_closed.store(true, Ordering::SeqCst);

        Ok(())
    }
    pub fn create<TEventHandler: Fn(&WindowEvent) -> () + Sync + Send + 'static>(
        &mut self,
        state_arc: Arc<Mutex<AppState>>,
        show: bool,
        event_handler: TEventHandler,
    ) -> JoinHandle<()> {
        let state = state_arc.clone();
        let inner_arc = self.inner.clone();
        let (sender, receiver) = channel();

        let t = thread::spawn(move || unsafe {
            let mut inner = inner_arc.lock();
            let instance = winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut());
            let c_name = CString::new(inner.title.clone().as_str()).unwrap();

            let class = WNDCLASSA {
                hInstance: instance,
                lpszClassName: c_name.as_ptr(),
                lpfnWndProc: Some(window_cb),
                hbrBackground: CreateSolidBrush(inner.background_color as u32),
                ..WNDCLASSA::default()
            };

            RegisterClassA(&class);

            let mut exstyle = 0;
            let mut style = WS_OVERLAPPEDWINDOW;

            if inner.is_popup {
                exstyle = WS_EX_NOACTIVATE | WS_EX_TOPMOST;
                style = WS_POPUPWINDOW;
            }

            if !inner.border {
                style &= !WS_BORDER
            }

            let hwnd = CreateWindowExA(
                exstyle,
                c_name.as_ptr(),
                c_name.as_ptr(),
                style,
                inner.x,
                inner.y,
                inner.width,
                inner.height,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null_mut(),
            );

            sender.send(hwnd.into()).unwrap();

            let win: NativeWindow = hwnd.into();

            if show {
                win.show();
            }

            inner.native_window = Some(win);

            let font = inner.font.clone();
            let title = inner.title.clone();
            let font_size = inner.font_size;
            let background_color = inner.background_color;

            drop(inner);

            //TODO: make this cleaner
            #[cfg(target_os = "windows")]
            unsafe {
                use winapi::um::shellapi::SHAppBarMessage;
                use winapi::um::shellapi::ABE_TOP;
                use winapi::um::shellapi::ABM_NEW;
                use winapi::um::shellapi::APPBARDATA;
                use winapi::um::winuser::WM_APP;

                let mut appbar_data = APPBARDATA {
                    cbSize: 4 + 4 + 4 + 4 + 16 + 4,
                    hWnd: hwnd,
                    uCallbackMessage: WM_APP + 1,
                    uEdge: ABE_TOP,
                    ..Default::default()
                };

                SHAppBarMessage(ABM_NEW, &mut appbar_data as *mut APPBARDATA);
            }

            message_loop::start(move |msg| {
                if let Some(msg) = msg {
                    if msg.message == WM_IDENT {
                        let window: NativeWindow = hwnd.into();
                        let state = state.lock();
                        let display_id = fail_with!(window.get_display(), true).id;
                        let display = state.displays.iter().find(|d| d.id == display_id).unwrap();

                        let hdc = GetDC(hwnd);
                        let api = Api {
                            hdc: hdc as i32,
                            window: window.clone(),
                            display: display.clone(),
                            background_color,
                        };
                        let msg = *(msg.wParam as *const WindowMsg);

                        if msg.code == WM_PAINT {
                            let mut paint = PAINTSTRUCT::default();

                            BeginPaint(hwnd, &mut paint);

                            let mut logfont = LOGFONTA::default();
                            let mut font_name: [i8; 32] = [0; 32];

                            for (i, byte) in CString::new(font.as_str())
                                .unwrap()
                                .as_bytes()
                                .iter()
                                .enumerate()
                            {
                                font_name[i] = *byte as i8;
                            }

                            logfont.lfHeight = font_size;
                            logfont.lfFaceName = font_name;

                            let font = CreateFontIndirectA(&logfont);
                            SelectObject(hdc, font as *mut c_void);

                            SetBkColor(hdc, background_color as u32);

                            event_handler(&WindowEvent::Draw {
                                display: &display,
                                id: window.id,
                                state: &state,
                                api,
                            });

                            DeleteObject(font as *mut c_void);
                            EndPaint(hwnd, &paint);
                        } else if msg.code == WM_LBUTTONDOWN {
                            let mut point = POINT::default();
                            GetCursorPos(&mut point);
                            let win_rect = window.get_rect().unwrap();

                            event_handler(&WindowEvent::Click {
                                id: window.id,
                                x: point.x - win_rect.left,
                                y: point.y - win_rect.top,
                                state: &state,
                                display: &display,
                            });
                        } else if msg.code == WM_CLOSE {
                            let name = CString::new(title.clone()).unwrap();

                            UnregisterClassA(
                                name.as_ptr(),
                                winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null_mut()),
                            );

                            event_handler(&WindowEvent::Close {
                                id: window.id,
                                display: &display,
                            });
                        } else if msg.code == WM_CREATE {
                            event_handler(&WindowEvent::Create {
                                id: window.id,
                                display: &display,
                            });
                        } else if msg.code == WM_MOUSEMOVE {
                            let mut point = POINT::default();
                            GetCursorPos(&mut point);
                            let win_rect = window.get_rect().unwrap();

                            event_handler(&WindowEvent::MouseMove {
                                id: window.id,
                                display: &display,
                                api,
                                x: point.x - win_rect.left,
                                y: point.y - win_rect.top,
                            });
                        } else {
                            event_handler(&WindowEvent::Native {
                                msg,
                                display: &display,
                            });
                        }

                        ReleaseDC(hwnd, hdc);
                    }
                }

                true
            })
        });

        self.id = receiver.recv().unwrap();

        if self.refresh_rate > 0 {
            let id = self.id.clone();
            let refresh_rate = self.refresh_rate;
            let is_closed = self.is_closed.clone();

            //refresh thread
            thread::spawn(move || {
                while !is_closed.load(Ordering::SeqCst) {
                    sleep!(refresh_rate as u64);
                    let window: NativeWindow = id.into();
                    window.redraw();
                }
            });
        }

        t
    }
}
