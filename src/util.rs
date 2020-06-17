use winapi::shared::windef::HWND;
use winapi::um::winuser::GetWindowTextA;
use thiserror::Error;

pub fn get_title_of_window(window_handle: HWND) -> Result<String, WinApiResultError> {
    let mut buffer = [0;0x200];

    unsafe {
        winapi_nullable_to_result(GetWindowTextA(window_handle, buffer.as_mut_ptr(), buffer.len() as i32))?;
    };

    Ok(buffer
        .iter()
        .map(|byte| char::from(*byte as u8))
        .collect::<String>())
}

pub type WinApiResult<T> = Result<T, WinApiResultError>;

#[derive(Debug, Error)]
pub enum WinApiResultError {
    #[error("Windows Api errored and returned a value of {0}")]
    Err(i32),
    #[error("Windows Api errored and returned a null value")]
    Null
}

pub fn winapi_err_to_result<T>(input: T) -> WinApiResult<T> where T: PartialEq<i32> + Into<i32> {
    if input == 0 {
        Ok(input)
    } else {
        Err(WinApiResultError::Err(input.into()))
    }
}

pub fn winapi_ptr_to_result<T>(input: *mut T) -> WinApiResult<*mut T> {
    if input != std::ptr::null_mut() {
        Ok(input)
    } else {
        Err(WinApiResultError::Null)
    }
}

pub fn winapi_nullable_to_result<T>(input: T) -> WinApiResult<T> where T: PartialEq<i32> {
    if input != 0 {
        Ok(input)
    } else {
        Err(WinApiResultError::Null)
    }
}