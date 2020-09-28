use super::Renderer;
use crate::{system::SystemError, system::SystemResult, tile::Tile, tile_grid::TileGrid, CONFIG};
use winapi::{shared::windef::*, um::winuser::*};

#[derive(Default, Clone, Copy)]
pub struct WinRenderer;

impl Renderer for WinRenderer {
    fn render<TRenderer: Renderer>(&self, grid: &TileGrid<TRenderer>, tile: &Tile) -> SystemResult {
        let (padding, margin) = {
            let config = CONFIG.lock();

            (config.inner_gap, config.outer_gap)
        };
        let display_height = grid.display.working_area_height() - margin * 2 - padding * 2;
        let display_width = grid.display.working_area_width() - margin * 2 - padding * 2;
        let column_width = display_width / grid.columns;
        let row_height = display_height / grid.rows;
        let mut x = grid.display.working_area_left();
        let mut y = grid.display.working_area_top();
        let mut height = display_height;
        let mut width = display_width;

        if !grid.fullscreen {
            if let Some(column) = tile.column {
                width = column_width;
                x += column_width * (column - 1);

                if column > 1 {
                    width -= padding;
                    x += padding;
                }
            }

            if let Some(row) = tile.row {
                height = row_height;
                y = row_height * (row - 1);

                if row > 1 {
                    height -= padding;
                    y += padding;
                }
            }

            x += margin;
            x += padding;
            y += margin;
            y += padding;

            let (column_modifications, row_modifications) = grid.get_modifications(tile);

            if let Some(modifications) = column_modifications {
                let real_left = grid.percentage_to_real(modifications.0);
                let real_right = grid.percentage_to_real(modifications.1);

                x -= real_left;
                width += real_right + real_left;
            }
            if let Some(modifications) = row_modifications {
                let real_top = grid.percentage_to_real(modifications.0);
                let real_bottom = grid.percentage_to_real(modifications.1);

                y -= real_top;
                height += real_bottom + real_top;
            }

            // column to the right
            if let Some(modifications) = grid.get_column_modifications(tile.column.map(|x| x + 1)) {
                let real_left = grid.percentage_to_real(modifications.0);

                width -= real_left;
            }

            // row below
            if let Some(modifications) = grid.get_row_modifications(tile.row.map(|x| x + 1)) {
                let real_top = grid.percentage_to_real(modifications.0);

                height -= real_top;
            }

            // column to the left
            if let Some(modifications) = grid.get_column_modifications(tile.column.map(|x| x - 1)) {
                let real_right = grid.percentage_to_real(modifications.1);

                x += real_right;
                width -= real_right;
            }

            // row above
            if let Some(modifications) = grid.get_row_modifications(tile.row.map(|x| x - 1)) {
                let real_bottom = grid.percentage_to_real(modifications.1);

                y += real_bottom;
                height -= real_bottom;
            }
        } else {
            x += margin;
            x += padding;
            y += margin;
            y += padding;
        }

        let rule = tile.window.rule.clone().unwrap_or_default();
        let (remove_title_bar, use_border) = {
            let config = CONFIG.lock();

            (config.remove_title_bar, config.use_border)
        };

        let mut left = x;
        let mut right = x + width;
        let mut top = y;
        let mut bottom = y + height;

        unsafe {
            let border_width = GetSystemMetricsForDpi(SM_CXFRAME, grid.display.dpi);
            let border_height = GetSystemMetricsForDpi(SM_CYFRAME, grid.display.dpi);

            if rule.chromium || rule.firefox || !remove_title_bar {
                let caption_height = GetSystemMetricsForDpi(SM_CYCAPTION, grid.display.dpi);
                top += caption_height;
            } else {
                top -= border_height * 2;

                if use_border {
                    left += 1;
                    right -= 1;
                    top += 1;
                    bottom -= 1;
                }
            }

            if rule.firefox || rule.chromium || (!remove_title_bar && rule.has_custom_titlebar) {
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
