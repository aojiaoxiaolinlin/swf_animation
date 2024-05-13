use std::sync::Arc;

// use ruffle_render::backend::RenderBackend;

use ruffle_render::{backend::RenderBackend, commands::CommandList};

use crate::{display_object::stage::Stage, library::Library, tag_utils::SwfMovie};

pub struct UpdateContext<'a> {
    pub library: &'a mut Library,
    pub player_version: u8,
    // pub swf: &'a mut Arc<SwfMovie>,
    pub renderer: &'a mut dyn RenderBackend,
    pub forced_frame_rate: bool,
    pub frame_rate: &'a mut f64,
    pub stage: &'a mut Stage,
}
impl<'a> UpdateContext<'a> {
    pub fn set_root_movie(&mut self, movie: Arc<SwfMovie>) {
        if !self.forced_frame_rate {
            *self.frame_rate = movie.frame_rate().into();
        }
        dbg!(movie.version());
        dbg!(movie.width());
        dbg!(movie.height());

        self.stage.set_movie_size((
            movie.width().to_pixels() as u32,
            movie.height().to_pixels() as u32,
        ));
        self.stage.set_movie(movie.clone());
    }
}
pub struct RenderContext<'a> {
    pub renderer: &'a mut dyn RenderBackend,
    pub commands: CommandList,
}
