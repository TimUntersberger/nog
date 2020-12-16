use super::Renderer;
use crate::{
    config::Config, display::Display, system::NativeWindow, system::SystemError,
    system::SystemResult, tile_grid::TileGrid,
};
use winapi::{shared::windef::*, um::winuser::*};

#[derive(Default, Clone, Copy, Debug)]
pub struct WinRenderer;

impl Renderer for WinRenderer {
    fn render<TRenderer: Renderer>(
        &self,
        grid: &TileGrid<TRenderer>,
        window: &NativeWindow,
        config: &Config,
        display: &Display,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> SystemResult {
        let rule = window.rule.clone().unwrap_or_default();

        let mut left = x;
        let mut right = x + width;
        let mut top = y;
        let mut bottom = y + height;

        unsafe {
            let border_width = GetSystemMetricsForDpi(SM_CXFRAME, display.dpi);
            let border_height = GetSystemMetricsForDpi(SM_CYFRAME, display.dpi);

            if rule.chromium || rule.firefox || !config.remove_title_bar {
                let caption_height = GetSystemMetricsForDpi(SM_CYCAPTION, display.dpi);
                top += caption_height;
            } else {
                top -= border_height * 2;

                if config.use_border {
                    left += 1;
                    right -= 1;
                    top += 1;
                    bottom -= 1;
                }
            }

            if rule.firefox
                || rule.chromium
                || (!config.remove_title_bar && rule.has_custom_titlebar)
            {
                if rule.firefox {
                    left -= (border_width as f32 * 1.5) as i32;
                    right += (border_width as f32 * 1.5) as i32;
                    bottom += (border_height as f32 * 1.5) as i32;
                } else if rule.chromium {
                    top -= border_height / 2;
                    left -= border_width * 2;
                    right += border_width * 2;
                    bottom += border_height * 2;
                }
                left += border_width * 2;
                right -= border_width * 2;
                top += border_height * 2;
                bottom -= border_height * 2;
            } else {
                top += border_height * 2;
            }
        }

        let mut rect = RECT {
            left,
            right,
            top,
            bottom,
        };

        // println!("before {}", rect_to_string(rect));

        unsafe {
            AdjustWindowRectEx(
                &mut rect,
                window.style.bits() as u32,
                0,
                window.exstyle.bits() as u32,
            );
        }

        // println!("after {}", rect_to_string(rect));

        window
            .set_window_pos(rect.into(), None, Some(SWP_NOSENDCHANGING))
            .map_err(SystemError::DrawTile)
    }
}
