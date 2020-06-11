use crate::CONFIG;
use crate::task_bar;
use crate::app_bar;

use winapi::um::winuser::GetSystemMetrics;
use winapi::um::winuser::SM_CYSCREEN;
use winapi::um::winuser::SM_CXSCREEN;

// height of the "display"
// this is not the height of the real display
// it is the real display minus the appbar height
pub static mut HEIGHT: i32 = 0;
pub static mut WIDTH: i32 = 0;

pub unsafe fn init(){
    HEIGHT = GetSystemMetrics(SM_CYSCREEN) - app_bar::HEIGHT;
    WIDTH = GetSystemMetrics(SM_CXSCREEN);

    if !CONFIG.remove_title_bar {
        HEIGHT = HEIGHT + 9;
        WIDTH = WIDTH + 15;
    } 

    // +2 because the taskbar is apparently still on the screen when hidden haha
    let taskbar_is_visible = task_bar::Y + 2 + app_bar::HEIGHT < HEIGHT;

    println!("{}, {}", task_bar::Y, HEIGHT);

    if taskbar_is_visible && !CONFIG.remove_task_bar {
        HEIGHT = HEIGHT - task_bar::HEIGHT;
    }
}