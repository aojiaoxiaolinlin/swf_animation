mod character;
mod context;
mod library;
mod display_object;
mod string;
mod types;
pub mod tag_utils;

use std::fs::read;

use display_object::movie_clip::MovieClip;

fn main() {
    let data =
        read("D:\\Code\\Rust\\swf_animation\\desktop\\swf_files\\spirit2471src.swf").unwrap();
    let mut movie_clip = MovieClip::new(data);
    movie_clip.parse();
}
