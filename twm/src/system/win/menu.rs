use crate::{
    keybindings::{key::Key, modifier::Modifier},
    message_loop,
    system::win,
    system::NativeWindow,
    system::Rectangle,
    system::WindowId,
    util,
};
use num_traits::FromPrimitive;
use parking_lot::Mutex;
use std::{ffi::CString, slice::Iter, sync::Arc};
use winapi::{
    shared::{minwindef::*, windef::*},
    um::{errhandlingapi::*, libloaderapi::*, psapi::*, wingdi::*, winnt::*, winuser::*, fileapi::*, minwinbase::*},
};

#[derive(FromPrimitive)]
enum HotKeyKind {
    Quit = 1,
    DeleteWordBackward,
}

enum WindowCommand {
    Created = (WM_APP + 1) as isize,
    InputChanged
}

unsafe extern "system" fn window_cb(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_CREATE {
        let class = util::to_widestring("Edit");

        let textbox = CreateWindowExW(
            0,
            class.as_ptr(),
            std::ptr::null_mut(),
            WS_CHILD | WS_VISIBLE,
            15,
            15,
            670,
            30,
            hwnd,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        let mut logfont = LOGFONTA::default();
        let mut font_name: [i8; 32] = [0; 32];

        for (i, byte) in CString::new("Consolas")
            .unwrap()
            .as_bytes()
            .iter()
            .enumerate()
        {
            font_name[i] = *byte as i8;
        }

        logfont.lfHeight = 25;
        logfont.lfFaceName = font_name;

        let font = CreateFontIndirectA(&logfont);

        SendMessageA(textbox, WM_SETFONT, font as usize, 1);
        SetFocus(textbox);

        RegisterHotKey(hwnd, HotKeyKind::Quit as i32, 0, Key::Escape as u32);
        RegisterHotKey(
            hwnd,
            HotKeyKind::DeleteWordBackward as i32,
            Modifier::CONTROL.bits(),
            Key::Backspace as u32,
        );
        RegisterHotKey(
            hwnd,
            HotKeyKind::DeleteWordBackward as i32,
            Modifier::CONTROL.bits(),
            Key::W as u32,
        );

        PostMessageA(hwnd, WindowCommand::Created as u32, textbox as usize, 0);
    } else if msg == WM_COMMAND {
        let kind = HIWORD(w_param as u32);

        if kind == EN_CHANGE {
            PostMessageA(hwnd, WindowCommand::InputChanged as u32, 0, 0);
        }
    }
    DefWindowProcA(hwnd, msg, w_param, l_param)
}

pub struct Menu {
    tb_win_id: Arc<Mutex<WindowId>>,
    item_win_ids: Arc<Mutex<Vec<WindowId>>>,
    on_input_changed: Option<Arc<dyn Fn(String) -> ()>>
}

impl Menu {
    pub fn new() -> Self {
        Self {
            tb_win_id: Arc::new(Mutex::new(WindowId::default())),
            item_win_ids: Arc::new(Mutex::new(Vec::new())),
            on_input_changed: None
        }
    }
    pub fn on_input_changed(mut self, f: impl Fn(String) -> () + 'static) -> Self {
        self.on_input_changed = Some(Arc::new(f));
        self
    }
    pub fn create(&mut self) {
        unsafe {
            let instance = GetModuleHandleA(std::ptr::null_mut());
            let c_name = CString::new("Menu2").unwrap();

            let class = WNDCLASSA {
                hInstance: instance,
                lpszClassName: c_name.as_ptr(),
                lpfnWndProc: Some(window_cb),
                // hbrBackground: CreateSolidBrush(inner.background_color as u32),
                ..WNDCLASSA::default()
            };

            if RegisterClassA(&class) == 0 {
                UnregisterClassA(c_name.as_ptr(), instance);
                RegisterClassA(&class);
            }

            let height = 60;
            let width = 700;

            let hwnd = CreateWindowExA(
                0,
                c_name.as_ptr(),
                std::ptr::null_mut(),
                WS_POPUPWINDOW,
                0,
                0,
                width,
                height,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null_mut(),
            );

            let win: NativeWindow = hwnd.into();

            // center the window on its display
            if let Ok(display) = win.get_display() {
                let display_width = display.rect.width();
                let display_height = display.rect.height();
                let mut rect = Rectangle::default();

                rect.left = display_width / 2 - width / 2;
                rect.right = rect.left + width;

                rect.top = display_height / 2 - height / 2 - 200;
                rect.bottom = rect.top + height;

                win.set_window_pos(rect, None, None);
            }

            win.show();

            let tb = self.tb_win_id.clone();

            message_loop::start(|msg| {
                if let Some(msg) = msg {
                    if msg.message == WM_HOTKEY {
                        if GetForegroundWindow() != msg.hwnd {
                            return true;
                        }
                        if let Some(kind) = HotKeyKind::from_usize(msg.wParam) {
                            match kind {
                                HotKeyKind::Quit => {
                                    *tb.lock() = Default::default();
                                    CloseWindow(msg.hwnd);
                                    for id in self.item_win_ids.lock().iter() {
                                        CloseWindow(id.0 as HWND);
                                    }
                                    self.item_win_ids.lock().clear();
                                    return false;
                                }
                                HotKeyKind::DeleteWordBackward => {
                                    let tb = tb.lock().clone();
                                    let buffer_len = GetWindowTextLengthA(tb.into()) + 1;
                                    let mut buffer = vec![0; buffer_len as usize];
                                    GetWindowTextA(
                                        tb.into(),
                                        buffer.as_mut_ptr(),
                                        buffer_len
                                    );
                                    let mut new_end = buffer_len - 1;
                                    for i in (0..buffer_len - 1).rev() {
                                        new_end = i;
                                        // if char is space
                                        if buffer[i as usize] == 32 {
                                            break;
                                        }
                                    }
                                    let buffer: Vec<i8> = buffer
                                        .into_iter()
                                        .take(new_end as usize)
                                        .chain(std::iter::once('\0' as i8))
                                        .collect();

                                    SetWindowTextA(tb.into(), buffer.as_ptr());
                                    SendMessageA(tb.into(), EM_SETSEL as u32, (new_end + 1) as usize, (new_end + 1) as isize);
                                }
                            }
                        }
                    } else if msg.message == WindowCommand::Created as u32 {
                        let tb_hwnd = msg.wParam as i32;
                        *tb.lock() = tb_hwnd.into();
                    } else if msg.message == WindowCommand::InputChanged as u32 {
                        println!("CHANGED");
                        if let Some(cb) = self.on_input_changed.as_ref() {
                            let tb = tb.lock().clone();
                            let buffer_len = GetWindowTextLengthA(tb.into()) + 1;
                            let mut buffer = vec![0; buffer_len as usize];
                            GetWindowTextA(
                                tb.into(),
                                buffer.as_mut_ptr(),
                                buffer_len
                            );
                            let text = util::bytes_to_string(&buffer);
                            cb(text);
                            // let mut data: WIN32_FIND_DATAA = Default::default();
                            // let search_handle = FindFirstFileA(CString::new("firefox").unwrap().as_ptr(), &mut data);
                            // // dbg!(data.alt);
                            // // todo!("Show suggestions");
                        }
                    }
                }
                true
            });
        }
    }
}

#[test]
pub fn test_menu() {
    Menu::new()
        .on_input_changed(|new_input| {
            println!("CHANGED: {}", new_input);
        })
        .create();
}
