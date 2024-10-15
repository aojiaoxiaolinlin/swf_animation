use std::{cell::RefCell, sync::Arc};

use player_core::{
    display_object::{
        movie_clip::{MovieClip, MovieClipFlags},
        DisplayObject, DisplayObjectBase, TDisplayObject,
    },
    library::MovieLibrary,
    tag_utils,
};
use ruffle_render::{blend::ExtendedBlendMode, transform::Transform};
use swf::{CharacterId, Rectangle, Shape};
use tracing::debug;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();

    let data = std::fs::read("core/tests/swfs/spirit2159src.swf").unwrap();
    let swf_movie = tag_utils::SwfMovie::from_data(&data[..]).unwrap();
    let frame_rate = swf_movie.header().frame_rate();

    let mut movie_clip = MovieClip::new(Arc::new(swf_movie));
    let mut movie_library = MovieLibrary::new();

    movie_clip.parse_swf(&mut movie_library);
    movie_clip.set_is_root(true);

    let mut vector_animation =
        VectorAnimation::new("spirit2159src".to_owned(), frame_rate.to_f32() as u16);

    debug!("frame rate: {}", vector_animation.frame_rate);
    for _ in 0..5 {
        movie_clip.enter_frame(&mut movie_library);
    }

    if let Some(mc) = movie_clip.query_movie_clip("_mc") {
        let mut goto_frame = 0;
        while goto_frame <= mc.borrow().total_frames() {
            let mut frames = 0;
            mc.borrow_mut().set_is_root(true);
            mc.borrow_mut()
                .goto_frame(&mut movie_library, goto_frame, true);
            mc.borrow_mut()
                .raw_container()
                .render_list()
                .iter()
                .for_each(|d| {
                    if let Some(child) = d.borrow().as_movie_clip() {
                        frames = child.total_frames();
                    }
                });
            debug!("frames: {}", frames);
            for _ in 0..frames {
                mc.borrow_mut().enter_frame(&mut movie_library);
                let render_list = mc.borrow().raw_container().render_list();
                cover_rend_list_to_time_line(
                    &mut vector_animation.animations,
                    render_list,
                    Transform::default(),
                );
            }
            goto_frame += 10;
        }
    }
}

struct VectorAnimation {
    name: String,
    frame_rate: u16,
    animations: Vec<Animation>,
}
impl VectorAnimation {
    fn new(name: String, frame_rate: u16) -> Self {
        Self {
            name,
            frame_rate,
            animations: vec![],
        }
    }
}

struct Animation {
    time_line: Vec<TimeLineObject>,
    current_frame: u16,
    playing: bool,
    paused: bool,
    frame_rate: f32,
    current_label: Option<String>,
}
impl Animation {
    fn new() -> Self {
        Self {
            time_line: vec![],
            current_frame: 0,
            playing: false,
            paused: false,
            frame_rate: 24.0,
            current_label: None,
        }
    }
}
struct TimeLineObject {
    place_frame: u16,
    id: CharacterId,
    shape: Shape,
    bounds: Rectangle<f32>,
    name: Option<String>,
    animation: Vec<(u16, Transform)>,
}

fn cover_rend_list_to_time_line(
    animations: &mut Vec<Animation>,
    render_list: Arc<Vec<Arc<RefCell<DisplayObject>>>>,
    transform: Transform,
) {
    for render_object in render_list.iter() {
        if let Some(movie_clip) = render_object.borrow().as_movie_clip() {
            cover_rend_list_to_time_line(
                animations,
                movie_clip.raw_container().render_list().clone(),
                transform.clone() * movie_clip.transform(),
            );
        }
        if let Some(shape) = render_object.borrow().as_graphic() {}
    }
}
