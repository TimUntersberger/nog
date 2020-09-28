use crate::{
    bar::Bar,
    system::DisplayId,
    system::{api, Rectangle},
    task_bar,
};
use std::cmp::Ordering;
use task_bar::{Taskbar, TaskbarPosition};

#[derive(Default, Debug, Clone)]
pub struct Display {
    pub id: DisplayId,
    pub dpi: u32,
    pub rect: Rectangle,
    pub taskbar: Option<Taskbar>,
    pub appbar: Option<Bar>,
}

impl Display {
    pub fn height(&self) -> i32 {
        self.rect.height()
    }
    pub fn width(&self) -> i32 {
        self.rect.width()
    }
    pub fn is_primary(&self) -> bool {
        self.rect.left == 0 && self.rect.top == 0
    }
    pub fn get_rect(&self) -> Rectangle {
        api::get_display_rect(self.id)
    }
    pub fn working_area_height(&self) -> i32 {
        let tb_height = self
            .taskbar
            .clone()
            .map(|tb| match tb.position {
                // Should probably handle the error at some point instead of just unwraping
                TaskbarPosition::Top | TaskbarPosition::Bottom => {
                    tb.window.get_rect().unwrap().height()
                }
                _ => 0,
            })
            .unwrap_or(0);

        self.height()
            - if CONFIG.lock().remove_task_bar {
                0
            } else {
                tb_height
            }
            - if CONFIG.lock().display_app_bar {
                CONFIG.lock().bar.height
            } else {
                0
            }
    }
    pub fn get_taskbar_position(&self) -> TaskbarPosition {
        // Should probably handle the error at some point instead of just unwraping
        let task_bar_rect = self.taskbar.clone().unwrap().window.get_rect().unwrap();
        if self.rect.left == task_bar_rect.left
            && self.rect.top == task_bar_rect.top
            && self.rect.bottom == task_bar_rect.bottom
        {
            TaskbarPosition::Left
        } else if self.rect.right == task_bar_rect.right
            && self.rect.top == task_bar_rect.top
            && self.rect.bottom == task_bar_rect.bottom
        {
            TaskbarPosition::Right
        } else if self.rect.left == task_bar_rect.left
            && self.rect.top == task_bar_rect.top
            && self.rect.right == task_bar_rect.right
        {
            TaskbarPosition::Top
        } else {
            TaskbarPosition::Bottom
        }
    }
    pub fn working_area_width(&self) -> i32 {
        let tb_width = self
            .taskbar
            .clone()
            .map(|tb| match tb.position {
                // Should probably handle the error at some point instead of just unwraping
                TaskbarPosition::Left | TaskbarPosition::Right => {
                    tb.window.get_rect().unwrap().width()
                }
                _ => 0,
            })
            .unwrap_or(0);

        self.width()
            - if CONFIG.lock().remove_task_bar {
                0
            } else {
                tb_width
            }
    }
    pub fn working_area_top(&self) -> i32 {
        let mut offset = self
            .taskbar
            .clone()
            .map(|t| match t.position {
                // Should probably handle the error at some point instead of just unwraping
                TaskbarPosition::Top => t.window.get_rect().unwrap().height(),
                _ => 0,
            })
            .unwrap_or(0);

        let config = CONFIG.lock();

        if config.display_app_bar {
            offset += config.bar.height;
        }

        self.rect.top + offset
    }
    pub fn working_area_left(&self) -> i32 {
        let offset = self
            .taskbar
            .clone()
            .map(|t| match t.position {
                // Should probably handle the error at some point instead of just unwraping
                TaskbarPosition::Left => t.window.get_rect().unwrap().width(),
                _ => 0,
            })
            .unwrap_or(0);

        self.rect.left + offset
    }
    pub fn new(id: DisplayId) -> Self {
        let mut display = Display::default();

        display.dpi = api::get_display_dpi(id);
        display.id = id;
        display.rect = display.get_rect();

        display
    }
}

pub fn init(multi_monitor: bool) -> Vec<Display> {
    let mut displays = api::get_displays();

    if multi_monitor {
        displays = displays
            .iter_mut()
            .filter(|d| d.is_primary())
            .map(|d| d.to_owned())
            .collect();
    }

    displays.sort_by(|x, y| {
        let ordering = y.rect.left.cmp(&x.rect.left);

        if ordering == Ordering::Equal {
            return y.rect.top.cmp(&x.rect.top);
        }

        ordering
    });

    displays

    // task_bar::update_task_bars();
}

pub fn with_display_by<TF, TCb, TReturn>(f: TF, cb: TCb) -> TReturn
where
    TF: Fn(&&mut Display) -> bool,
    TCb: Fn(Option<&mut Display>) -> TReturn,
{
    cb(DISPLAYS.lock().iter_mut().find(f))
}

pub fn get_primary_display() -> Display {
    with_display_by(|d| d.is_primary(), |d| d.unwrap().clone())
}

pub fn with_display_by_idx<TCb, TReturn>(idx: i32, cb: TCb) -> TReturn
where
    TCb: Fn(Option<&mut Display>) -> TReturn,
{
    let mut displays = DISPLAYS.lock();

    let x: usize = if idx == -1 {
        0
    } else {
        std::cmp::max(displays.len() - (idx as usize), 0)
    };

    cb(displays.get_mut(x))
}
