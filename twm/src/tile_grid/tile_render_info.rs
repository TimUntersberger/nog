use crate::system::NativeWindow;

pub struct TileRenderInfo {
    pub window: NativeWindow,
    pub x: u32,
    pub y: u32,
    pub height: u32,
    pub width: u32,
    pub debug_id: usize,
    pub debug_size: u32,
    pub debug_order: u32,
}
