mod character;
mod container;
mod context;
mod display_object;
mod drawing;
mod library;
pub mod player;
mod stage;
mod tag_utils;
#[cfg(test)]
mod tests {
    use std::{fs::read, sync::Arc};

    use url::Url;

    use crate::{
        container::TDisplayObjectContainer,
        display_object::{movie_clip::MovieClip, TDisplayObject},
        library::MovieLibrary,
        tag_utils::SwfMovie,
    };

    #[test]
    fn test_movie_clip() {
        let path = if cfg!(target_os = "windows") {
            "D:\\Code\\Rust\\swf_animation\\desktop\\swf_files\\spirit2471src.swf"
        } else {
            "/home/intasect/study/Rust/swf_animation/desktop/swf_files/spirit2471src.swf"
        };
        let swf_movie = SwfMovie::from_path(path, None).unwrap();
        let mut movie_clip = MovieClip::new(Arc::new(swf_movie));
        let mut movie_library = MovieLibrary::new();
        movie_clip.set_name(Some("root".to_string()));
        movie_clip.load_swf(&mut movie_library);

        let ident = 1;
        println!(
            "name = {},total_frames = {}
            length = {}",
            movie_clip.name().unwrap(),
            movie_clip.total_frames,
            movie_clip.movie().data().len()
        );
        movie_clip.run_frame_internal(&mut movie_library);
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
