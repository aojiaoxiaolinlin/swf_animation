use std::{cell::RefCell, collections::HashMap, fs::File, io::BufReader, sync::Arc};

use player_core::{
    display_object::{DisplayObject, TDisplayObject},
    tag_utils::{self, SwfMovie},
};
use ruffle_render::transform::Transform;
use swf::{CharacterId, Encoding, Header, PlaceObjectAction, Rectangle, Shape, SwfStr, Tag};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();

    let path = "core/tests/swfs/spirit2159src.swf";
    let reader = BufReader::new(File::open(path).unwrap());

    let swf_buf = swf::decompress_swf(reader).unwrap();
    let swf = swf::parse_swf(&swf_buf).unwrap();
    info!("SWF 文件版本: {}", swf.header.version());
    info!("SWF 文件有: {} 帧", swf.header.num_frames());
    info!("该 SWF 文件包含 {} 个标签", swf.tags.len());

    let total_frames = swf.header.swf_header().num_frames;
    let encoding_for_version = SwfStr::encoding_for_version(swf.header.version());

    let mut target_tags = Vec::new();
    let mut sprites = Vec::new();
    get_targe_mc(
        &mut target_tags,
        swf.tags,
        "_mc",
        encoding_for_version,
        &mut sprites,
    );

    let mut new_header = swf::Header {
        num_frames: total_frames,
        ..swf.header.swf_header().clone()
    };
    let mut i = 3;
    'sprites: for (name, id) in sprites {
        if i >= 1 {
            i -= 1;
            continue;
        }
        info!("转换目标标签：{}, sprite id: {}", name, id);
        let mut new_tags = target_tags.clone();
        new_tags.retain(|tag| {
            if let swf::Tag::ShowFrame = tag {
                return false;
            }
            true
        });

        for tag in &mut target_tags {
            if let Tag::DefineSprite(sprite) = tag {
                if sprite.id == id {
                    new_header.num_frames = sprite.num_frames;
                    dbg!(sprite.num_frames);
                    // new_tags.extend(sprite.tags.);
                    new_tags.append(&mut sprite.tags.clone());

                    let file =
                        std::fs::File::create("core/tests/swfs/new/spirit2159src4.swf").unwrap();
                    let writer = std::io::BufWriter::new(file);
                    swf::write_swf(&new_header, &new_tags, writer).unwrap();
                    break 'sprites;
                }
            }
        }
    }

    // let swf_movie = tags_to_swf_movie(&target_tags, swf.header.swf_header());

    // let frame_rate = swf_movie.header().frame_rate();
    // info!("帧率: {}", frame_rate);

    // let mut movie_clip = MovieClip::new(Arc::new(swf_movie));
    // let mut movie_library = MovieLibrary::new();

    // movie_clip.parse_swf(&mut movie_library);
    // movie_clip.set_is_root(true);

    // let mut vector_animation =
    //     VectorAnimation::new("spirit2159src".to_owned(), frame_rate.to_f32() as u16);
    // let mut base_animations: HashMap<CharacterId, Animation> = HashMap::new();
    // for _ in 0..5 {
    //     movie_clip.enter_frame(&mut movie_library);
    // }
    // let animation_name = [
    //     "STF", "OTF", "WAI", "ATT", "UDA", "BTD", "MIS", "MGS", "MGF", "MGE", "DEA", "OWK",
    // ];
    // if let Some(mc) = movie_clip.query_movie_clip("_mc") {
    //     let mut goto_frame = 0;
    //     while goto_frame <= mc.borrow().total_frames()
    //         && goto_frame < animation_name.len() as u16 * 10
    //     {
    //         let mut frames = 0;
    //         mc.borrow_mut().set_is_root(true);
    //         mc.borrow_mut()
    //             .goto_frame(&mut movie_library, goto_frame, true);
    //         mc.borrow_mut()
    //             .raw_container()
    //             .render_list()
    //             .iter()
    //             .for_each(|d| {
    //                 if let Some(child) = d.borrow().as_movie_clip() {
    //                     frames = child.total_frames();
    //                 }
    //             });
    //         debug!("GOTO:{}, frames: {}", goto_frame, frames);
    //         for _ in 0..frames {
    //             mc.borrow_mut().enter_frame(&mut movie_library);
    //             let render_list = mc.borrow().raw_container().render_list();
    //             let mut animation = Animation::default();
    //             cover_rend_list_to_time_line(
    //                 &mut base_animations,
    //                 &mut animation,
    //                 render_list,
    //                 Transform::default(),
    //             );
    //         }
    //         goto_frame += 10;
    //     }
    //     vector_animation.base_animations = base_animations;
    //     info!("animation: {:?}", vector_animation.base_animations.len());
    // }
}

fn tags_to_swf_movie(tags: &[Tag], swf_header: &Header) -> SwfMovie {
    let path = "core/tests/swfs/new/spirit2159src.swf";
    let file = std::fs::File::create(path).unwrap();
    let writer = std::io::BufWriter::new(file);
    swf::write_swf(swf_header, &tags, writer).unwrap();

    let data = std::fs::read(path).unwrap();
    tag_utils::SwfMovie::from_data(&data[..]).unwrap()
}

struct VectorAnimation {
    name: String,
    frame_rate: u16,
    base_animations: HashMap<CharacterId, Animation>,
    animation_ids: Vec<CharacterId>,
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

fn get_targe_mc<'a>(
    new_tags: &mut Vec<swf::Tag<'a>>,
    tags: Vec<Tag<'a>>,
    target: &str,
    encoding_for_version: &'static Encoding,
    sprites: &mut Vec<(String, CharacterId)>,
) {
    for tag in tags {
        let res = match tag {
            swf::Tag::ShowFrame => swf::Tag::ShowFrame,
            swf::Tag::DefineBitsJpeg2 { id, jpeg_data } => {
                swf::Tag::DefineBitsJpeg2 { id, jpeg_data }
            }
            swf::Tag::DefineBitsJpeg3(define_bits_jpeg3) => {
                swf::Tag::DefineBitsJpeg3(define_bits_jpeg3)
            }
            swf::Tag::DefineShape(shape) => swf::Tag::DefineShape(shape),
            swf::Tag::DefineSprite(sprite) => swf::Tag::DefineSprite(sprite),
            swf::Tag::Metadata(metadata) => swf::Tag::Metadata(metadata),
            swf::Tag::PlaceObject(place_object) => match place_object.action {
                swf::PlaceObjectAction::Place(id) => {
                    if let Some(name) = place_object.name {
                        if target == name.to_str_lossy(&encoding_for_version) {
                            // 删除new_tags中id相同的ShowFrame标签
                            new_tags.retain(|tag| {
                                if let swf::Tag::ShowFrame = tag {
                                    return false;
                                }
                                true
                            });
                            // 从new_tags中找到id相同的DefineSprite，并删除
                            let target = new_tags
                                .iter_mut()
                                .position(|tag| {
                                    if let swf::Tag::DefineSprite(sprite) = tag {
                                        return id == sprite.id;
                                    }
                                    false
                                })
                                .map(|index| new_tags.remove(index));
                            if let Some(target) = target {
                                if let swf::Tag::DefineSprite(sprite) = target {
                                    get_target_label(
                                        new_tags,
                                        sprite.tags,
                                        sprites,
                                        encoding_for_version,
                                    );
                                    return; // 返回，不再处理后续的PlaceObject标签
                                }
                            }
                        }
                    }
                    swf::Tag::PlaceObject(place_object)
                }
                _ => swf::Tag::PlaceObject(place_object),
            },
            swf::Tag::FileAttributes(file_attributes) => swf::Tag::FileAttributes(file_attributes),
            swf::Tag::Unknown {
                tag_code: _,
                data: _,
            } => {
                continue;
            }
            _ => {
                continue;
            }
        };

        new_tags.push(res);
    }
}

fn get_target_label<'a>(
    new_tags: &mut Vec<swf::Tag<'a>>,
    tags: Vec<Tag<'a>>,
    sprites: &mut Vec<(String, CharacterId)>,
    encoding_for_version: &'static Encoding,
) {
    let mut label_name = String::from("");
    for tag in tags {
        let res = match tag {
            swf::Tag::ShowFrame => swf::Tag::ShowFrame,
            swf::Tag::DefineBitsJpeg2 { id, jpeg_data } => {
                swf::Tag::DefineBitsJpeg2 { id, jpeg_data }
            }
            swf::Tag::DefineBitsJpeg3(define_bits_jpeg3) => {
                swf::Tag::DefineBitsJpeg3(define_bits_jpeg3)
            }
            swf::Tag::DefineShape(shape) => swf::Tag::DefineShape(shape),
            swf::Tag::DefineSprite(sprite) => swf::Tag::DefineSprite(sprite),
            swf::Tag::Metadata(metadata) => swf::Tag::Metadata(metadata),
            swf::Tag::SetBackgroundColor(color) => swf::Tag::SetBackgroundColor(color),
            swf::Tag::PlaceObject(place_object) => match place_object.action {
                PlaceObjectAction::Place(id) => {
                    sprites.push((label_name.clone(), id));

                    swf::Tag::PlaceObject(place_object)
                }
                _ => swf::Tag::PlaceObject(place_object),
            },
            swf::Tag::RemoveObject(remove_object) => swf::Tag::RemoveObject(remove_object),
            swf::Tag::FileAttributes(file_attributes) => swf::Tag::FileAttributes(file_attributes),
            swf::Tag::FrameLabel(frame_label) => {
                label_name = frame_label
                    .label
                    .to_str_lossy(encoding_for_version)
                    .to_string();
                swf::Tag::FrameLabel(frame_label)
            }
            swf::Tag::Unknown {
                tag_code: _,
                data: _,
            } => {
                continue;
            }
            _ => {
                continue;
            }
        };
        new_tags.push(res);
    }
}
