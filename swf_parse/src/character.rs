use crate::display_object::{graphic::Graphic, morph_shape::MorphShape, movie_clip::MovieClip};


pub enum Character {
    Graphic(Graphic),
    MorphShape(MorphShape),
    MovieClip(MovieClip),
}