mod character;
mod display_object;
mod library;
mod movie_clip;
mod container;
mod graphic;
mod morph_shape;
#[cfg(test)]
mod tests {
    use std::fs::read;

    use crate::{library::MovieLibrary, movie_clip::MovieClip};

    #[test]
    fn test_movie_clip() {
        let data = if cfg!(target_os = "windows") {
            read("D:\\Code\\Rust\\swf_animation\\desktop\\swf_files\\spirit2471src.swf").unwrap()
        } else {
            read("/home/intasect/study/Rust/swf_animation/desktop/swf_files/spirit2471src.swf")
                .unwrap()
        };
        let swf_buf = swf::decompress_swf(&data[..]).unwrap();
        let parse_swf = swf::parse_swf(&swf_buf).unwrap();
        let mut movie_clip = MovieClip::new(parse_swf.header);
        let mut movie_library = MovieLibrary::new();
        movie_clip.load_swf(parse_swf.tags, &mut movie_library);
    }
}
