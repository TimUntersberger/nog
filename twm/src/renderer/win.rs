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
        {
            let mut rect = RECT {
                left: x,
                right: x + width,
                top: y,
                bottom: y + height,
            };

            let window_display = window.get_display()?;

            // If the display dpi is different, then first position the window on the correct display.
            // Otherwise the extended rect calculations below will not be correct
            if (display.dpi != window_display.dpi) {
                window.set_window_pos(rect.into(), None, Some(SWP_NOSENDCHANGING))?
            }

            // Windows adds an invisible border around the windows, so we need to take that into
            // account when positioning the windows
            let window_rect = window.get_rect()?;
            let frame_rect = window.get_extended_rect()?;
            let left_margin = frame_rect.left - window_rect.left;
            let right_margin = frame_rect.right - window_rect.right;
            let bottom_margin = frame_rect.bottom - window_rect.bottom;
            rect.left -= left_margin;
            rect.right -= right_margin;
            rect.bottom -= bottom_margin;

            window
                .set_window_pos(rect.into(), None, Some(SWP_NOSENDCHANGING))
        }.map_err(SystemError::DrawTile)
    }
}
