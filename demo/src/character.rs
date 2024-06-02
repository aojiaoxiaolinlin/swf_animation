use crate::{graphic::Graphic, movie_clip::MovieClip};

#[derive(Clone)]
pub enum Character {
    MovieClip(MovieClip),
    Graphic(Graphic),
}