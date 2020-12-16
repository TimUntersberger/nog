use super::{DisplayId, Rectangle, SystemError, SystemResult, WindowId};
use crate::{
    display::Display, util, window::gwl_ex_style::GwlExStyle, window::gwl_style::GwlStyle, Rule,
};
use log::error;
use thiserror::Error;
use winapi::{
    shared::{minwindef::*, windef::*},
    um::{errhandlingapi::*, psapi::*, winnt::*, winuser::*, *},
};

pub mod api;
pub mod win_event_listener;

pub const BIN_NAME: &'static str = "nog.exe";

impl From<HWND> for WindowId {
    fn from(val: HWND) -> Self {
        Self(val as i32)
    }
}

impl Into<HWND> for WindowId {
    fn into(self) -> HWND {
        self.0 as HWND
    }
}

impl PartialEq<HWND> for WindowId {
    fn eq(&self, other: &HWND) -> bool {
        self.0 == (*other) as i32
    }
}

impl From<HMONITOR> for DisplayId {
    fn from(val: HMONITOR) -> Self {
        Self(val as i32)
    }
}

impl Into<HMONITOR> for DisplayId {
    fn into(self) -> HMONITOR {
        self.0 as HMONITOR
    }
}

impl From<HMONITOR> for Display {
    fn from(val: HMONITOR) -> Self {
        Self::new(val.into())
    }
}

impl PartialEq<i32> for Display {
    fn eq(&self, other: &i32) -> bool {
        self.id == *other
    }
}

#[derive(Error, Debug)]
pub enum WinError {
    #[error("Winapi return value is null")]
    Null,
    #[error("Winapi return value is false")]
    Bool,
}

pub type WinResult<T = ()> = Result<T, WinError>;

impl From<RECT> for Rectangle {
    fn from(rect: RECT) -> Self {
        Self {
            left: rect.left,
            right: rect.right,
            top: rect.top,
            bottom: rect.bottom,
        }
    }
}

impl Into<RECT> for Rectangle {
    fn into(self) -> RECT {
        RECT {
            left: self.left,
            right: self.right,
            top: self.top,
            bottom: self.bottom,
        }
    }
}

fn bool_to_result(v: BOOL) -> WinResult {
    if v == 0 {
        Err(WinError::Bool)
    } else {
        Ok(())
    }
}

fn nullable_to_result<T: PartialEq<i32>>(v: T) -> WinResult<T> {
    if v != 0 {
        Ok(v)
    } else {
        Err(WinError::Null)
    }
}

fn lresult_to_result(v: LRESULT) -> WinResult<LRESULT> {
    Ok(v)
}

#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub title: String,
    pub maximized: bool,
    pub rule: Option<Rule>,
    pub style: GwlStyle,
    pub exstyle: GwlExStyle,
    pub original_style: GwlStyle,
    pub original_rect: Rectangle,
}

impl PartialEq<i32> for Window {
    fn eq(&self, other: &i32) -> bool {
        self.id == *other
    }
}

impl From<WindowId> for Window {
    fn from(val: WindowId) -> Self {
        let mut window = Window::new();
        window.id = val;
        window
    }
}

impl From<HWND> for Window {
    fn from(val: HWND) -> Self {
        let mut window = Window::new();
        window.id = val.into();
        window
    }
}

impl Window {
    pub fn is_hidden(&self) -> bool {
        unsafe { IsWindowVisible(self.id.into()) == 0 }
    }
    pub fn is_visible(&self) -> bool {
        !self.is_hidden()
    }
    pub fn should_manage(&self) -> bool {
        self.original_style.contains(GwlStyle::CAPTION)
            && !self.exstyle.contains(GwlExStyle::DLGMODALFRAME)
    }
    pub fn remove_title_bar(&mut self, use_border: bool) -> SystemResult {
        let rule = self.rule.clone().unwrap_or_default();
        if !rule.chromium && !rule.firefox {
            self.style.remove(GwlStyle::CAPTION);
            self.style.remove(GwlStyle::THICKFRAME);
        }
        if use_border {
            self.style.insert(GwlStyle::BORDER);
        }
        self.update_style()
            .map(|_| {})
            .map_err(SystemError::Unknown)
    }
    pub fn get_display(&self) -> WinResult<Display> {
        unsafe {
            nullable_to_result(MonitorFromWindow(self.id.into(), MONITOR_DEFAULTTONULL).into())
        }
    }
    pub fn get_foreground_window() -> SystemResult<Window> {
        unsafe {
            nullable_to_result(GetForegroundWindow().into())
                .map_err(SystemError::GetForegroundWindow)
        }
    }
    pub fn get_class_name(&self) -> WinResult<String> {
        let mut buffer = [0; 0x200];

        unsafe {
            nullable_to_result(GetClassNameA(
                self.id.into(),
                buffer.as_mut_ptr(),
                buffer.len() as i32,
            ))
            .map(|_| util::bytes_to_string(&buffer))
        }
    }
    pub fn get_parent_window(&self) -> WinResult<WindowId> {
        unsafe { nullable_to_result(GetParent(self.id.into()).into()) }
    }
    pub fn get_style(&self) -> WinResult<GwlStyle> {
        unsafe {
            nullable_to_result(GetWindowLongA(self.id.into(), GWL_STYLE))
                .map(|x| GwlStyle::from_bits_unchecked(x as u32 as i32))
        }
    }
    pub fn get_ex_style(&self) -> WinResult<GwlExStyle> {
        unsafe {
            nullable_to_result(GetWindowLongA(self.id.into(), GWL_EXSTYLE))
                .map(|x| GwlExStyle::from_bits_unchecked(x as u32 as i32))
        }
    }
    pub fn get_title(&self) -> WinResult<String> {
        let mut buffer = [0; 0x200];

        unsafe {
            nullable_to_result(GetWindowTextA(
                self.id.into(),
                buffer.as_mut_ptr(),
                buffer.len() as i32,
            ))
            .map(|_| util::bytes_to_string(&buffer))
        }
    }
    pub fn get_rect(&self) -> WinResult<Rectangle> {
        unsafe {
            let mut temp = RECT::default();
            nullable_to_result(GetWindowRect(self.id.into(), &mut temp)).map(|_| temp.into())
        }
    }
    pub fn is_window(&self) -> bool {
        unsafe { IsWindow(self.id.into()) != 0 }
    }
    pub fn reset_style(&mut self) {
        self.style = self.original_style;
    }
    pub fn update_style(&self) -> WinResult<i32> {
        unsafe {
            nullable_to_result::<i32>(SetWindowLongA(self.id.into(), GWL_STYLE, self.style.bits()))
        }
    }
    /// This could error if the window is already in the foreground
    pub fn to_foreground(&self, topmost: bool) -> WinResult {
        self.set_window_pos(
            Rectangle::default(),
            Some(if topmost { HWND_TOPMOST } else { HWND_TOP }),
            Some(SWP_NOMOVE | SWP_NOSIZE),
        )
    }
    pub fn remove_topmost(&self) -> WinResult {
        self.set_window_pos(
            Rectangle::default(),
            Some(HWND_NOTOPMOST),
            Some(SWP_NOMOVE | SWP_NOSIZE),
        )
    }
    pub fn set_window_pos(
        &self,
        rect: Rectangle,
        order: Option<HWND>,
        flags: Option<u32>,
    ) -> WinResult {
        unsafe {
            bool_to_result(SetWindowPos(
                self.id.into(),
                order.unwrap_or(std::ptr::null_mut()),
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                flags.unwrap_or_default(),
            ))
        }
    }
    fn reset_pos(&self) -> WinResult {
        self.set_window_pos(self.original_rect, None, None)
    }
    pub fn get_process_name(&self) -> String {
        self.get_process_path()
            .split('\\')
            .last()
            .unwrap()
            .to_string()
    }
    // TODO: rewrite
    pub fn get_process_path(&self) -> String {
        let mut buffer = [0; 0x200];

        unsafe {
            let mut process_id = 0;
            GetWindowThreadProcessId(self.id.into(), &mut process_id);
            let process_handle = processthreadsapi::OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                0,
                process_id,
            );

            if process_handle as i32 == 0 {
                error!("winapi: {}", GetLastError());
            }
            if GetModuleFileNameExA(
                process_handle,
                std::ptr::null_mut(),
                buffer.as_mut_ptr(),
                buffer.len() as u32,
            ) == 0
            {
                error!("winapi: {}", GetLastError());
            };
        }

        util::bytes_to_string(&buffer)
    }
    pub fn new() -> Self {
        Self {
            id: 0.into(),
            title: String::from(""),
            maximized: false,
            rule: None,
            style: GwlStyle::default(),
            exstyle: GwlExStyle::default(),
            original_style: GwlStyle::default(),
            original_rect: Rectangle::default(),
        }
    }
    pub fn cleanup(&mut self) -> SystemResult {
        self.reset_style();
        self.update_style().map_err(SystemError::CleanupWindow)?;
        self.reset_pos().map_err(SystemError::CleanupWindow)?;

        if self.maximized {
            self.maximize()?;
        }

        Ok(())
    }
    pub fn show(&self) {
        unsafe {
            ShowWindow(self.id.into(), SW_SHOW);
        }
    }
    pub fn hide(&self) {
        unsafe {
            ShowWindow(self.id.into(), SW_HIDE);
        }
    }
    pub fn close(&self) -> SystemResult {
        unsafe {
            lresult_to_result(SendMessageA(self.id.into(), WM_SYSCOMMAND, SC_CLOSE, 0))
                .map(|_| {})
                .map_err(SystemError::CloseWindow)
        }
    }
    pub fn focus(&self) -> SystemResult {
        unsafe {
            bool_to_result(SetForegroundWindow(self.id.into()))
                .map(|_| {})
                .map_err(SystemError::FocusWindow)
        }
    }
    pub fn redraw(&self) -> SystemResult {
        unsafe {
            lresult_to_result(SendMessageA(self.id.into(), WM_PAINT, 0, 0))
                .map(|_| {})
                .map_err(SystemError::RedrawWindow)
        }
    }
    pub fn init(&mut self) -> SystemResult {
        self.original_style = self.get_style().map_err(SystemError::Init)?;
        if self.original_style.contains(GwlStyle::MAXIMIZE) {
            self.restore().map_err(SystemError::Init)?;
            self.maximized = true;
            self.original_style.remove(GwlStyle::MAXIMIZE);
        }
        self.style = self.original_style;
        self.exstyle = self.get_ex_style().map_err(SystemError::Init)?;
        self.original_rect = self.get_rect().map_err(SystemError::Init)?;

        Ok(())
    }
    fn restore(&self) -> WinResult {
        unsafe {
            lresult_to_result(SendMessageA(self.id.into(), WM_SYSCOMMAND, SC_RESTORE, 0))
                .map(|_| {})
        }
    }
    pub fn minimize(&self) -> SystemResult {
        unsafe {
            lresult_to_result(SendMessageA(self.id.into(), WM_SYSCOMMAND, SC_MINIMIZE, 0))
                .map(|_| {})
                .map_err(SystemError::MinimizeWindow)
        }
    }
    pub fn maximize(&self) -> SystemResult {
        unsafe {
            lresult_to_result(SendMessageA(self.id.into(), WM_SYSCOMMAND, SC_MAXIMIZE, 0))
                .map(|_| {})
                .map_err(SystemError::MaximizeWindow)
        }
    }
}
