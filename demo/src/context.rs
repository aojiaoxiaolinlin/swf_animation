use ruffle_render::{backend::{BitmapCacheEntry, RenderBackend}, commands::CommandList, transform::TransformStack};

use crate::library::{MovieLibrary, MovieLibrarySource};

pub struct RenderContext<'a> {
    pub renderer: &'a mut dyn RenderBackend,

    pub commands: CommandList,

    pub cache_draws: &'a mut Vec<BitmapCacheEntry>,

    pub transform_stack: &'a mut TransformStack,

    pub is_offscreen: bool,

    pub use_bitmap_cache: bool,

    pub library: &'a mut MovieLibrary
}