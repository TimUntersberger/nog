use super::Renderer;
use crate::{
    config::Config, display::Display, system::SystemError, system::SystemResult, tile::Tile,
    tile_grid::TileGrid,
};
use winapi::{shared::windef::*, um::winuser::*};

#[derive(Default, Clone, Copy, Debug)]
pub struct WinRenderer;

impl Renderer for WinRenderer {
    fn render<TRenderer: Renderer>(
        &self,
        grid: &TileGrid<TRenderer>,
        tile: &Tile,
        config: &Config,
        display: &Display,
    ) -> SystemResult {
        let display_height =
            display.working_area_height(config) - config.outer_gap * 2 - config.inner_gap * 2;
        let display_width =
            display.working_area_width(config) - config.outer_gap * 2 - config.inner_gap * 2;
        let column_width = display_width / grid.columns;
        let row_height = display_height / grid.rows;
        let mut x = display.working_area_left();
        let mut y = display.working_area_top(config);
        let mut height = display_height;
        let mut width = display_width;

        if !grid.fullscreen {
            if let Some(column) = tile.column {
                width = column_width;
                x += column_width * (column - 1);

                if column > 1 {
                    width -= config.inner_gap;
                    x += config.inner_gap;
                }
            }

            if let Some(row) = tile.row {
                height = row_height;
                y = row_height * (row - 1);

                if row > 1 {
                    height -= config.inner_gap;
                    y += config.inner_gap;
                }
            }

            x += config.outer_gap;
            x += config.inner_gap;
            y += config.outer_gap;
            y += config.inner_gap;

            let (column_modifications, row_modifications) = grid.get_modifications(tile);

            if let Some(modifications) = column_modifications {
                let real_left = self.percentage_to_real(modifications.0, display, config);
                let real_right = self.percentage_to_real(modifications.1, display, config);

                x -= real_left;
                width += real_right + real_left;
            }
            if let Some(modifications) = row_modifications {
                let real_top = self.percentage_to_real(modifications.0, display, config);
                let real_bottom = self.percentage_to_real(modifications.1, display, config);

                y -= real_top;
                height += real_bottom + real_top;
            }

            // column to the right
            if let Some(modifications) = grid.get_column_modifications(tile.column.map(|x| x + 1)) {
                let real_left = self.percentage_to_real(modifications.0, display, config);

                width -= real_left;
            }

            // row below
            if let Some(modifications) = grid.get_row_modifications(tile.row.map(|x| x + 1)) {
                let real_top = self.percentage_to_real(modifications.0, display, config);

                height -= real_top;
            }

            // column to the left
            if let Some(modifications) = grid.get_column_modifications(tile.column.map(|x| x - 1)) {
                let real_right = self.percentage_to_real(modifications.1, display, config);

                x += real_right;
                width -= real_right;
            }

            // row above
            if let Some(modifications) = grid.get_row_modifications(tile.row.map(|x| x - 1)) {
                let real_bottom = self.percentage_to_real(modifications.1, display, config);

                y += real_bottom;
                height -= real_bottom;
            }
        } else {
            x += config.outer_gap;
            x += config.inner_gap;
            y += config.outer_gap;
            y += config.inner_gap;
        }

        let rule = tile.window.rule.clone().unwrap_or_default();

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
                tile.window.style.bits() as u32,
                0,
                tile.window.exstyle.bits() as u32,
            );
        }

        // println!("after {}", rect_to_string(rect));

        tile.window
            .set_window_pos(rect.into(), None, Some(SWP_NOSENDCHANGING))
            .map_err(SystemError::DrawTile)
    }
}
