use std::sync::Arc;

// use ruffle_render::backend::RenderBackend;

use ruffle_render::{backend::RenderBackend, commands::CommandList};

use crate::{display_object::{movie_clip::MovieClip, stage::Stage, DisplayObject, TDisplayObject}, library::Library, tag_utils::SwfMovie};

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
        dbg!(movie.width().to_pixels());
        dbg!(movie.height().to_pixels());

        self.stage.set_movie_size((
            movie.width().to_pixels() as u32,
            movie.height().to_pixels() as u32,
        ));
        self.stage.set_movie(movie.clone());

        let mut root: DisplayObject = MovieClip::player_root_movie(movie.clone()).into();

        root.set_depth(0);
        root.set_default_root_name()
    }
}
pub struct RenderContext<'a> {
    pub renderer: &'a mut dyn RenderBackend,
    pub commands: CommandList,
}
