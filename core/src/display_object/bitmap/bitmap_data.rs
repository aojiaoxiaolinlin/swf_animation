use ruffle_render::bitmap::{BitmapHandle, PixelRegion, SyncHandle};

use crate::display_object::DisplayObjectWeak;
pub struct Color(u32);

impl From<u32> for Color {
    fn from(i: u32) -> Self {
        Color(i)
    }
}

impl From<Color> for u32 {
    fn from(c: Color) -> Self {
        c.0
    }
}
pub struct BitmapDataWrapper {
    pixels: Vec<Color>,

    width: u32,
    height: u32,
    transparency: bool,

    disposed: bool,

    bitmap_handle: Option<BitmapHandle>,

    display_objects: Vec<DisplayObjectWeak>,

    dirty_state: DirtyState,
}
impl BitmapDataWrapper {
    pub fn new_with_pixels(
        width: u32,
        height: u32,
        transparency: bool,
        pixels: Vec<Color>,
    ) -> Self {
        Self {
            pixels,
            width,
            height,
            transparency,
            disposed: false,
            bitmap_handle: None,
            display_objects: Vec::new(),
            dirty_state: DirtyState::Clean,
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Clone, Debug)]
enum DirtyState {
    // Both the CPU and GPU pixels are up to date. We do not need to wait for any syncs to complete
    Clean,

    // The CPU pixels have been modified, and need to be synced to the GPU via `update_dirty_texture`
    CpuModified(PixelRegion),

    // The GPU pixels have been modified, and need to be synced to the CPU via `BitmapDataWrapper::sync`
    GpuModified(Box<dyn SyncHandle>, PixelRegion),
}
