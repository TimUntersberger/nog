use crate::{system::SystemResult, tile::Tile, tile_grid::TileGrid};

pub use win::WinRenderer as NativeRenderer;

pub mod win;

pub trait Renderer {
    fn render<TRenderer: Renderer>(&self, grid: &TileGrid<TRenderer>, tile: &Tile) -> SystemResult;
}
