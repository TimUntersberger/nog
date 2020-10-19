use crate::{
    config::Config, display::Display, system::SystemResult, tile_grid::TileGrid, system::NativeWindow,
};

pub use win::WinRenderer as NativeRenderer;

pub mod win;

pub trait Renderer {
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
    ) -> SystemResult;
    /// Converts the percentage to the real pixel value of the current display
    fn percentage_to_real(&self, p: i32, display: &Display, config: &Config) -> i32 {
        display.working_area_height(config) / 100 * p
    }
}
