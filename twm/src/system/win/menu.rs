use crate::{
    keybindings::{key::Key, modifier::Modifier},
    message_loop,
    system::win,
    system::NativeWindow,
    system::Rectangle,
    system::WindowId,
    util,
    window::Window,
    window::WindowEvent,
    AppState,
};
use num_traits::FromPrimitive;
use parking_lot::Mutex;
use std::{ffi::CString, slice::Iter, sync::atomic::AtomicBool, sync::atomic::Ordering, sync::Arc};
use winapi::{
    shared::{minwindef::*, windef::*},
    um::{
        errhandlingapi::*, fileapi::*, libloaderapi::*, minwinbase::*, psapi::*, wingdi::*,
        winnt::*, winuser::*,
    },
};

#[derive(FromPrimitive)]
enum HotKeyKind {
    Quit = 1,
    DeleteWordBackward,
    Execute,
}

enum WindowCommand {
    Created = (WM_APP + 1) as isize,
    InputChanged,
    LostFocus,
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
        RegisterHotKey(hwnd, HotKeyKind::Execute as i32, 0, Key::Enter as u32);
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
            PostMessageA(hwnd as HWND, WindowCommand::InputChanged as u32, 0, 0);
        } else if kind == EN_KILLFOCUS {
            PostMessageA(hwnd as HWND, WindowCommand::LostFocus as u32, 0, 0);
        }
    }
    DefWindowProcA(hwnd, msg, w_param, l_param)
}

#[derive(Default)]
pub struct MenuState {
    input: String,
    dirty: bool,
    items: Vec<String>,
}

impl MenuState {
    pub fn set_items(&mut self, items: Vec<String>) {
        self.dirty = true;
        self.items = items;
    }
}

fn create_menu_item_window(parent: WindowId, x: i32, y: i32, height: i32, width: i32) -> Window {
    Window::new()
        .with_parent(parent)
        .with_is_popup(true)
        .with_background_color(0x000000)
        .with_size(width, height)
        .with_pos(x, y)
}

pub struct Menu {
    tb_win_id: Arc<Mutex<WindowId>>,
    menu_win_id: Arc<Mutex<WindowId>>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    item_height: i32,
    y_offset: i32,
    max_result_count: usize,
    item_windows: Arc<Mutex<Vec<Window>>>,
    on_input_changed: Option<Arc<dyn Fn(&mut MenuState) -> () + Send + Sync>>,
    on_execute: Option<Arc<dyn Fn(&MenuState) -> () + Send + Sync>>,
    state: Arc<Mutex<MenuState>>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            tb_win_id: Arc::new(Mutex::new(WindowId::default())),
            menu_win_id: Arc::new(Mutex::new(WindowId::default())),
            max_result_count: 10,
            x: 0,
            y: 0,
            item_height: 20,
            height: 60,
            width: 700,
            y_offset: 200,
            item_windows: Arc::new(Mutex::new(Vec::new())),
            on_input_changed: None,
            on_execute: None,
            state: Arc::new(Mutex::new(MenuState::default())),
        }
    }
    pub fn on_input_changed(
        mut self,
        f: impl Fn(&mut MenuState) -> () + 'static + Send + Sync,
    ) -> Self {
        self.on_input_changed = Some(Arc::new(f));
        self
    }
    pub fn on_execute(mut self, f: impl Fn(&MenuState) -> () + 'static + Send + Sync) -> Self {
        self.on_execute = Some(Arc::new(f));
        self
    }
    pub fn close(&self) {
        unsafe {
            CloseWindow((*self.menu_win_id.lock()).into());
            CloseWindow((*self.tb_win_id.lock()).into());
        }
        for win in self.item_windows.lock().iter() {
            win.close().unwrap();
        }
        self.item_windows.lock().clear();
    }
    pub fn create(&mut self, state_arc: Arc<Mutex<AppState>>) {
        let item_windows = self.item_windows.clone();

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

            let (display_width, display_height) = {
                let state = state_arc.lock();
                let display = state.get_current_display();
                (display.rect.width(), display.rect.height())
            };

            self.x = display_width / 2 - self.width / 2;
            self.y = display_height / 2 - self.height / 2 - self.y_offset;

            let hwnd = CreateWindowExA(
                0,
                c_name.as_ptr(),
                std::ptr::null_mut(),
                WS_POPUPWINDOW,
                self.x,
                self.y,
                self.width,
                self.height,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null_mut(),
            );

            *self.menu_win_id.lock() = hwnd.into();

            let win: NativeWindow = hwnd.into();

            win.show();

            let tb = self.tb_win_id.clone();
            let menu = self.menu_win_id.clone();
            let menu_id = menu.lock().clone();

            *self.item_windows.lock() = vec![Window::default(); self.max_result_count];

            for i in 0..self.max_result_count {
                let state = self.state.clone();
                let item_windows = self.item_windows.clone();
                let state_arc = state_arc.clone();
                let x = self.x;
                let y = self.y;
                let height = self.item_height;
                let width = self.width;

                std::thread::spawn(move || {
                    let mut window = create_menu_item_window(menu_id, x, y, height, width);

                    window.create(state_arc.clone(), false, move |event| {
                        match event {
                            WindowEvent::Draw { api, .. } => {
                                api.set_text_color(0x00ffffff);
                                api.write_text(
                                    &state.lock().items.get(i).cloned().unwrap_or_default(),
                                    0,
                                    0,
                                    true,
                                    false,
                                );
                            }
                            _ => {}
                        }

                        Ok(())
                    });

                    item_windows.lock()[i] = window;
                });
            }

            message_loop::start(|msg| {
                if let Some(msg) = msg {
                    if msg.message == WM_HOTKEY {
                        if let Some(kind) = HotKeyKind::from_usize(msg.wParam) {
                            match kind {
                                HotKeyKind::Quit => {
                                    self.close();
                                    return false;
                                }
                                HotKeyKind::Execute => {
                                    self.close();
                                    if let Some(cb) = self.on_execute.as_ref().map(|x| x.clone()) {
                                        cb(&self.state.lock());
                                    }
                                    return false;
                                }
                                HotKeyKind::DeleteWordBackward => {
                                    let tb = tb.lock().clone();
                                    let buffer_len = GetWindowTextLengthA(tb.into()) + 1;
                                    let mut buffer = vec![0; buffer_len as usize];
                                    GetWindowTextA(tb.into(), buffer.as_mut_ptr(), buffer_len);
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
                                    SendMessageA(
                                        tb.into(),
                                        EM_SETSEL as u32,
                                        (new_end + 1) as usize,
                                        (new_end + 1) as isize,
                                    );
                                }
                            }
                        }
                    } else if msg.message == WindowCommand::Created as u32 {
                        let tb_hwnd = msg.wParam as i32;
                        *tb.lock() = tb_hwnd.into();
                    } else if msg.message == WindowCommand::LostFocus as u32 {
                        *tb.lock() = Default::default();
                        CloseWindow(msg.hwnd);
                        for win in self.item_windows.lock().iter() {
                            win.close().unwrap();
                        }
                        self.item_windows.lock().clear();
                        return false;
                    } else if msg.message == WindowCommand::InputChanged as u32 {
                        let tb = tb.lock().clone();
                        let buffer_len = GetWindowTextLengthA(tb.into()) + 1;
                        let mut buffer = vec![0; buffer_len as usize];
                        GetWindowTextA(tb.into(), buffer.as_mut_ptr(), buffer_len);
                        let text = util::bytes_to_string(&buffer);
                        let mut state = self.state.lock();
                        state.input = text;

                        if let Some(cb) = self.on_input_changed.as_ref().map(|x| x.clone()) {
                            let prev_items_size = state.items.len();
                            cb(&mut state);
                            let curr_items_size = state.items.len();
                            if state.dirty {
                                for i in 0..curr_items_size {
                                    let window_id = self.item_windows.lock()[i].id;
                                    let x = self.x;
                                    let y = self.y + self.height + (self.item_height * (i as i32));
                                    let height = self.item_height;
                                    let width = self.width;

                                    std::thread::spawn(move || {
                                        MoveWindow(window_id.into(), x, y, width, height, 0);
                                        ShowWindow(window_id.into(), SW_SHOWNOACTIVATE);
                                    });
                                }

                                if prev_items_size > curr_items_size {
                                    for i in curr_items_size..prev_items_size {
                                        let window_id = self.item_windows.lock()[i].id;
                                        std::thread::spawn(move || {
                                            ShowWindow(window_id.into(), SW_HIDE);
                                        });
                                    }
                                }

                                state.dirty = false;
                            }
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
    let app_state = AppState::default();
    let commands: Vec<String> = (0..10).map(|i| format!("x {}", i)).collect();

    Menu::new()
        .on_input_changed(move |state| {
            dbg!(&state.input);
            if state.input == "" {
                state.set_items(Vec::new());
            } else {
                let new_items = commands
                    .iter()
                    .filter(|s| s.starts_with(&state.input))
                    .map(|x| x.to_string())
                    .collect();

                state.set_items(new_items);
            }
        })
        .on_execute(|state| {
            if state.input.starts_with("$ ") {
                println!("Executing nogscript {}", &state.input[2..]);
            }
        })
        .create(Arc::new(Mutex::new(app_state)));
}
