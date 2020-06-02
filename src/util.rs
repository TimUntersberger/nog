use winapi::shared::windef::HWND;
use winapi::um::winuser::GetWindowTextA;

pub unsafe fn get_title_of_window(window_handle: HWND) -> Option<String> {
    let mut buffer = [0;0x200];
    let result = GetWindowTextA(window_handle, buffer.as_mut_ptr(), buffer.len() as i32);

    return match result {
        0 => None,
        _ => Some(buffer
                .iter()
                .map(|byte| char::from(*byte as u8))
                .collect::<String>())
    };
}