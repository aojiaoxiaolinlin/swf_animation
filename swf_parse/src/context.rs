
// use ruffle_render::backend::RenderBackend;

use ruffle_render::{backend::RenderBackend, commands::CommandList};

use crate::library::MovieLibrary;

pub struct UpdateContext<'a> {
    pub player_version: u8,
    pub library: &'a mut MovieLibrary,
    // pub renderer: &'a mut dyn RenderBackend,
}
impl<'a> UpdateContext<'a> {
    pub fn new(
        player_version: u8,
        library: &'a mut MovieLibrary,
        // renderer: &'a mut dyn RenderBackend,
    ) -> Self {
        Self {
            player_version,
            library,
            // renderer,
        }
    }
    pub fn library_mut(&mut self) -> &mut MovieLibrary {
        self.library
    }
}
pub struct RenderContext<'a> {
    pub renderer: &'a mut dyn RenderBackend,
    pub library: &'a MovieLibrary,
    pub commands: CommandList,
}
