use std::sync::Arc;

// use ruffle_render::backend::RenderBackend;

use ruffle_render::{backend::RenderBackend, commands::CommandList};

use crate::{library::Library, tag_utils::SwfMovie};

pub struct UpdateContext<'a>{
    pub library:Library,
    pub player_version: u8,
    // pub swf: &'a mut Arc<SwfMovie>,
    pub renderer: &'a mut dyn RenderBackend,
}

pub struct RenderContext<'a>{
    pub renderer: &'a mut dyn RenderBackend,
    pub commands:CommandList,
}
