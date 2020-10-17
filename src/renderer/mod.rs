use crate::{
    config::Config, display::Display, system::SystemResult, tile::Tile, tile_grid::TileGrid,
};

pub use win::WinRenderer as NativeRenderer;

pub mod win;

pub trait Renderer {
    fn render<TRenderer: Renderer>(
        &self,
        grid: &TileGrid<TRenderer>,
        tile: &Tile,
        config: &Config,
        display: &Display,
    ) -> SystemResult;
    /// Converts the percentage to the real pixel value of the current display
    fn percentage_to_real(&self, p: i32, display: &Display, config: &Config) -> i32 {
        display.working_area_height(config) / 100 * p
    }
}
