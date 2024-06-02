use crate::movie_clip::MovieClip;

#[derive(Clone)]
pub struct ChildContainer {
    render_list: Vec<MovieClip>,
}

impl ChildContainer {
    pub fn new() -> Self {
        Self {
            render_list: Vec::new(),
        }
    }
}