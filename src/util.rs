use core::fmt::Debug;
use thiserror::Error;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::wingdi::GetBValue;
use winapi::um::wingdi::GetGValue;
use winapi::um::wingdi::GetRValue;
use winapi::um::wingdi::RGB;
use winapi::um::winuser::{GetClassNameA, GetWindowTextA};

pub fn get_title_of_window(window_handle: HWND) -> Result<String, WinApiResultError> {
    let mut buffer = [0; 0x200];

    unsafe {
        winapi_nullable_to_result(GetWindowTextA(
            window_handle,
            buffer.as_mut_ptr(),
            buffer.len() as i32,
        ))?;
    };

    Ok(bytes_to_string(&buffer))
}

pub fn bytes_to_string(buffer: &[i8]) -> String {
    buffer
        .iter()
        .take_while(|b| **b != 0)
        .map(|byte| char::from(*byte as u8))
        .collect::<String>()
}

pub fn get_class_name_of_window(window_handle: HWND) -> Result<String, WinApiResultError> {
    let mut buffer = [0; 0x200];

    unsafe {
        winapi_nullable_to_result(GetClassNameA(
            window_handle,
            buffer.as_mut_ptr(),
            buffer.len() as i32,
        ))?;
    };

    Ok(buffer
        .iter()
        .take_while(|b| **b != 0)
        .map(|byte| char::from(*byte as u8))
        .collect::<String>())
}

pub type WinApiResult<T> = Result<T, WinApiResultError>;

#[derive(Debug, Error)]
pub enum WinApiResultError {
    #[error("Windows Api errored and returned a value of {0}")]
    Err(i32),
    #[error("Windows Api errored and returned a null value")]
    Null,
}

#[allow(dead_code)]
pub fn winapi_err_to_result<T>(input: T) -> WinApiResult<T>
where
    T: PartialEq<i32> + Into<i32>,
{
    if input == 0 {
        Ok(input)
    } else {
        Err(WinApiResultError::Err(input.into()))
    }
}

pub fn winapi_ptr_to_result<T>(input: *mut T) -> WinApiResult<*mut T> {
    if !input.is_null() {
        Ok(input)
    } else {
        Err(WinApiResultError::Null)
    }
}

pub fn winapi_nullable_to_result<T>(input: T) -> WinApiResult<T>
where
    T: PartialEq<i32>,
{
    if input != 0 {
        Ok(input)
    } else {
        Err(WinApiResultError::Null)
    }
}

pub fn to_widestring(string: &str) -> Vec<u16> {
    string.encode_utf16().chain(std::iter::once(0)).collect()
}

#[allow(dead_code)]
pub fn rect_to_string(rect: RECT) -> String {
    format!(
        "RECT(left: {}, right: {}, top: {}, bottom: {})",
        rect.left, rect.right, rect.top, rect.bottom
    )
}

pub fn scale_color(color: i32, factor: f64) -> i32 {
    let mut blue = GetBValue(color as u32);
    let mut green = GetGValue(color as u32);
    let mut red = GetRValue(color as u32);

    blue = (blue as f64 * factor).round() as u8;
    green = (green as f64 * factor).round() as u8;
    red = (red as f64 * factor).round() as u8;

    RGB(red, green, blue) as i32
}
