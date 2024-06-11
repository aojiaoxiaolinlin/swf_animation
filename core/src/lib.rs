mod character;
mod container;
mod display_object;
mod library;
pub mod player;
mod stage;
mod context;
mod drawing;
#[cfg(test)]
mod tests {
    use std::fs::read;

    use crate::{
        container::TDisplayObjectContainer,
        display_object::{movie_clip::MovieClip, TDisplayObject},
        library::MovieLibrary,
    };

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
        movie_clip.set_name(Some("root".to_string()));
        movie_clip.load_swf(parse_swf.tags, &mut movie_library);

        let ident = 1;
        println!("{},{}", movie_clip.name().unwrap(), movie_clip.total_frames);
        show_display_object(ident, movie_clip);
    }

    fn show_display_object(ident: i32, movie_clip: MovieClip) {
        movie_clip.iter_render_list().for_each(|mut x| {
            if let Some(movie_clip) = x.as_movie() {
                for _ in 0..ident {
                    print!("  ");
                }
                println!(
                    "name: {},total frame: {},child num: {}",
                    movie_clip.name().unwrap(),
                    movie_clip.total_frames,
                    movie_clip.clone().iter_render_list().count()
                );
                show_display_object(ident + 1, movie_clip.clone());
            } else {
                for _ in 0..ident {
                    print!("  ");
                }
                println!("{}", x.name().unwrap());
            }
        });
    }
}
