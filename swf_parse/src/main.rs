
use std::fs::read;

use swf_parse::display_object::movie_clip::MovieClip;

fn main() {
    let data =
        read("D:\\Code\\Rust\\swf_animation\\desktop\\swf_files\\spirit2471src.swf").unwrap();
    let mut movie_clip = MovieClip::new(data);
    movie_clip.parse();
}
