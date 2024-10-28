use std::{cell::RefCell, collections::HashMap, sync::Arc};

use player_core::{
    display_object::{movie_clip::MovieClip, DisplayObject, TDisplayObject},
    library::MovieLibrary,
    tag_utils,
};
use ruffle_render::transform::Transform;
use swf::{CharacterId, Rectangle, Shape};
use tracing::{debug, info};
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
    let mut base_animations: HashMap<CharacterId, Animation> = HashMap::new();
    debug!("frame rate: {}", vector_animation.frame_rate);
    for _ in 0..5 {
        movie_clip.enter_frame(&mut movie_library);
    }
    let animation_name = [
        "STF", "OTF", "WAI", "ATT", "UDA", "BTD", "MIS", "MGS", "MGF", "MGE", "DEA", "OWK",
    ];
    if let Some(mc) = movie_clip.query_movie_clip("_mc") {
        let mut goto_frame = 0;
        while goto_frame <= mc.borrow().total_frames()
            && goto_frame < animation_name.len() as u16 * 10
        {
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
            debug!("GOTO:{}, frames: {}", goto_frame, frames);
            for _ in 0..frames {
                mc.borrow_mut().enter_frame(&mut movie_library);
                let render_list = mc.borrow().raw_container().render_list();
                let mut animation = Animation::default();
                cover_rend_list_to_time_line(
                    &mut base_animations,
                    &mut animation,
                    render_list,
                    Transform::default(),
                );
            }
            goto_frame += 10;
        }
        vector_animation.base_animations = base_animations;
        info!("animation: {:?}", vector_animation.base_animations.len());
    }
}

struct VectorAnimation {
    name: String,
    frame_rate: u16,
    base_animations: HashMap<CharacterId, Animation>, //
    animation_ids: Vec<CharacterId>,                  //
}

impl VectorAnimation {
    fn new(name: String, frame_rate: u16) -> Self {
        Self {
            name,
            frame_rate,
            base_animations: HashMap::new(),
            animation_ids: vec![],
        }
    }
}
#[derive(Default)]
struct Animation {
    id: CharacterId,
    name: String,
    time_line: Vec<TimeLineObject>,
    current_frame: u16,
    total_frames: u16,
    children: Vec<CharacterId>, //嵌套的动画
}
impl Animation {
    fn new(id: CharacterId, total_frames: u16) -> Self {
        Self {
            id,
            name: Default::default(),
            time_line: vec![],
            current_frame: 0,
            total_frames,
            children: vec![],
        }
    }
}
struct TimeLineObject {
    place_frame: u16,
    id: CharacterId,
    shape: Shape,
    bounds: Rectangle<f32>,
    name: Option<String>,
    transform: Transform,
}

fn cover_rend_list_to_time_line(
    base_animations: &mut HashMap<CharacterId, Animation>,
    animation: &mut Animation,
    render_list: Arc<Vec<Arc<RefCell<DisplayObject>>>>,
    transform: Transform,
) {
    for render_object in render_list.iter() {
        if let Some(movie_clip) = render_object.borrow().as_movie_clip() {
            let id = movie_clip.character_id();
            animation.children.push(id);
            let mut animation = Animation::new(id, movie_clip.total_frames());
            cover_rend_list_to_time_line(
                base_animations,
                &mut animation,
                movie_clip.raw_container().render_list().clone(),
                transform.clone() * movie_clip.transform(),
            );
            base_animations.entry(id).or_insert(animation);
        }
        if let Some(shape) = render_object.borrow().as_graphic() {}
    }
}
