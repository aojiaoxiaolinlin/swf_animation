use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::BufWriter,
    path::PathBuf,
};

use swf::{CharacterId, Depth, Encoding, Tag};

#[derive(Serialize, Deserialize, Debug)]
pub struct VectorAnimation {
    name: String,
    frame_rate: u16,
    base_animations: HashMap<CharacterId, Animation>,
    animations: HashMap<String, Animation>,
}

impl VectorAnimation {
    pub fn new(name: String, frame_rate: u16) -> Self {
        Self {
            name,
            frame_rate,
            base_animations: HashMap::new(),
            animations: HashMap::new(),
        }
    }
    pub fn add_animation(&mut self, label: &String, animation: Animation) {
        self.animations.insert(label.to_owned(), animation);
    }
    pub fn animation(&mut self, label: &String) -> &mut Animation {
        self.animations
            .entry(label.to_owned())
            .or_insert(Animation::default())
    }
    fn register_base_animation(&mut self, id: CharacterId, animation: Animation) {
        self.base_animations.insert(id, animation);
    }
}
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Animation {
    time_lines: BTreeMap<Depth, TimeLine>,
    total_frames: u16,
}
impl Animation {
    fn time_lines(&mut self) -> &mut BTreeMap<Depth, TimeLine> {
        &mut self.time_lines
    }
    fn time_line(&mut self, depth: Depth) -> &mut TimeLine {
        self.time_lines.get_mut(&depth).unwrap()
    }
    fn insert_or_get_time_line(&mut self, depth: Depth) -> &mut TimeLine {
        self.time_lines
            .entry(depth)
            .or_insert_with(|| TimeLine::new())
    }
    fn set_total_frame(&mut self, total_frame: u16) {
        self.total_frames = total_frame;
    }
}
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct TimeLine {
    frames: Vec<Frame>,
}
impl TimeLine {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn add_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }
    fn late_frame(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Frame {
    id: CharacterId,
    place_frame: u16,
    duration: u16,
    // children: Vec<CharacterId>, //嵌套的动画
    matrix: Matrix,
    color_transform: ColorTransform,
    blend_mode: BlendMode,
    // filters: Vec<Filter>,
}

impl Frame {
    pub fn new(id: CharacterId, place_frame: u16) -> Self {
        Self {
            id,
            place_frame,
            ..Default::default()
        }
    }
    // 用于补齐空帧
    fn space_frame(duration: u16) -> Self {
        Self {
            duration,
            ..Default::default()
        }
    }
    pub fn auto_add(&mut self) {
        self.duration += 1;
    }
}

// #[derive(Serialize, Deserialize, Debug)]
// pub enum Filter {
//     DropShadowFilter(Box<DropShadowFilter>),
//     BlurFilter(Box<BlurFilter>),
//     GlowFilter(Box<GlowFilter>),
//     BevelFilter(Box<BevelFilter>),
//     GradientGlowFilter(Box<GradientFilter>),
//     ConvolutionFilter(Box<ConvolutionFilter>),
//     ColorMatrixFilter(Box<ColorMatrixFilter>),
//     GradientBevelFilter(Box<GradientFilter>),
// }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DropShadowFilter {
    pub color: [u8; 4],
    pub blur_x: f32,
    pub blur_y: f32,
    pub angle: f32,
    pub distance: f32,
    pub strength: f32,
    pub flags: u8,
}

impl From<swf::DropShadowFilter> for DropShadowFilter {
    fn from(filter: swf::DropShadowFilter) -> Self {
        Self {
            color: [
                filter.color.r,
                filter.color.g,
                filter.color.b,
                filter.color.a,
            ],
            blur_x: filter.blur_x.to_f32(),
            blur_y: filter.blur_y.to_f32(),
            angle: filter.angle.to_f32(),
            distance: filter.distance.to_f32(),
            strength: filter.strength.to_f32(),
            flags: filter.flags.bits(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Matrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f32,
    pub ty: f32,
}
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ColorTransform {
    mult_color: [f32; 4],
    add_color: [f32; 4],
}
#[derive(Default, Serialize, Deserialize, Debug)]
pub enum BlendMode {
    #[default]
    Normal,
    Add,
    Multiply,
    Layer,
    Screen,
    Lighten,
    Darken,
    Difference,
    Subtract,
    Invert,
    Alpha,
    Erase,
    Overlay,
    HardLight,
}

impl BlendMode {
    fn from_swf_blend_mode(blend_mode: swf::BlendMode) -> Self {
        match blend_mode {
            swf::BlendMode::Normal => Self::Normal,
            swf::BlendMode::Add => Self::Add,
            swf::BlendMode::Multiply => Self::Multiply,
            swf::BlendMode::Layer => Self::Layer,
            swf::BlendMode::Screen => Self::Screen,
            swf::BlendMode::Lighten => Self::Lighten,
            swf::BlendMode::Darken => Self::Darken,
            swf::BlendMode::Difference => Self::Difference,
            swf::BlendMode::Subtract => Self::Subtract,
            swf::BlendMode::Invert => Self::Invert,
            swf::BlendMode::Alpha => Self::Alpha,
            swf::BlendMode::Erase => Self::Erase,
            swf::BlendMode::Overlay => Self::Overlay,
            swf::BlendMode::HardLight => Self::HardLight,
        }
    }
}

// 生成动画文件
pub fn generation_animation(
    tags: Vec<Tag>,
    file_name: &str,
    output: &PathBuf,
    frame_rate: u16,
    encoding_for_version: &'static Encoding,
) -> anyhow::Result<()> {
    let file_name: Vec<&str> = file_name.split(".").collect();
    let mut vector_animation =
        VectorAnimation::new(file_name.first().unwrap().to_string(), frame_rate);

    tags.iter().for_each(|tag| {
        if let Tag::DefineSprite(sprite) = tag {
            parse_sprite_animation(&mut vector_animation, sprite);
        }
    });

    parse_animation(&mut vector_animation, tags, encoding_for_version);

    // 清除空白帧
    clear_blank_frame(&mut vector_animation);
    // 写入animation.json文件
    let writer = BufWriter::new(File::create(output.join("animation.json"))?);

    serde_json::to_writer_pretty(writer, &vector_animation)?;

    Ok(())
}

fn parse_animation(
    vector_animation: &mut VectorAnimation,
    tags: Vec<Tag>,
    encoding_for_version: &'static Encoding,
) {
    let mut animation_name = String::from("");
    let mut current_frame = 0;
    for tag in tags {
        match tag {
            Tag::PlaceObject(place_object) => match place_object.action {
                swf::PlaceObjectAction::Place(id) => {
                    //  保证第一帧有标签区分不同的动画，如果没有设置则视为只有一个动画，将其命名为default
                    if &animation_name == "" {
                        animation_name = String::from("default");
                        vector_animation.add_animation(&animation_name, Animation::default());
                    }
                    add_time_line(
                        vector_animation.animation(&animation_name),
                        &place_object,
                        id,
                        current_frame,
                    );
                }
                swf::PlaceObjectAction::Modify => {
                    modify_at_depth(
                        vector_animation.animation(&animation_name),
                        &place_object,
                        current_frame,
                    );
                }
                swf::PlaceObjectAction::Replace(id) => {
                    replace_at_depth(
                        vector_animation
                            .animation(&animation_name)
                            .time_line(place_object.depth),
                        &place_object,
                        id,
                        current_frame,
                    );
                }
            },
            Tag::RemoveObject(remove_object) => {
                remove_at_depth(
                    vector_animation
                        .animation(&animation_name)
                        .time_line(remove_object.depth),
                );
            }
            Tag::ShowFrame => {
                current_frame += 1;
                vector_animation
                    .animation(&animation_name)
                    .time_lines()
                    .iter_mut()
                    .for_each(|(_, time_line)| {
                        let late_frame = time_line.late_frame();
                        late_frame.auto_add();
                    });
            }
            Tag::FrameLabel(frame_label) => {
                // 此时当前动画结束，开始下一个动画，记录总帧数
                if vector_animation.animations.len() > 0 {
                    vector_animation
                        .animation(&animation_name)
                        .set_total_frame(current_frame);
                }
                current_frame = 0;
                animation_name = frame_label.label.to_string_lossy(encoding_for_version);
                vector_animation.add_animation(&animation_name, Animation::default());
            }
            _ => {
                continue;
            }
        }
    }
    // 最后一个动画结束，记录总帧数
    vector_animation
        .animation(&animation_name)
        .set_total_frame(current_frame);
}

fn parse_sprite_animation(vector_animation: &mut VectorAnimation, sprite: &swf::Sprite) {
    let mut current_frame = 0;
    let mut animation = Animation::default();
    animation.set_total_frame(sprite.num_frames);
    for tag in sprite.tags.iter() {
        match tag {
            Tag::PlaceObject(place_object) => match place_object.action {
                swf::PlaceObjectAction::Place(id) => {
                    add_time_line(&mut animation, place_object, id, current_frame);
                }
                swf::PlaceObjectAction::Modify => {
                    modify_at_depth(&mut animation, place_object, current_frame);
                }
                swf::PlaceObjectAction::Replace(id) => {
                    replace_at_depth(
                        animation.time_line(place_object.depth),
                        place_object,
                        id,
                        current_frame,
                    );
                }
            },
            Tag::RemoveObject(remove_object) => {
                remove_at_depth(animation.time_line(remove_object.depth));
            }
            Tag::ShowFrame => {
                current_frame += 1;
                animation
                    .time_lines()
                    .iter_mut()
                    .for_each(|(_, time_line)| {
                        let late_frame = time_line.late_frame();
                        late_frame.auto_add();
                    });
            }
            _ => {
                continue;
            }
        }
    }
    vector_animation.register_base_animation(sprite.id, animation);
}

fn add_time_line(
    animation: &mut Animation,
    place_object: &Box<swf::PlaceObject>,
    id: CharacterId,
    current_frame: u16,
) {
    let timeline = animation.insert_or_get_time_line(place_object.depth);
    // 补齐空白帧
    let frame_delta = current_frame
        - timeline
            .frames
            .iter()
            .map(|frame| frame.duration)
            .sum::<u16>();
    if frame_delta > 0 {
        timeline.add_frame(Frame::space_frame(
            current_frame - timeline.frames.len() as u16,
        ));
    }
    let mut frame = Frame::new(id, current_frame);
    apply_place_object(&mut frame, place_object);
    timeline.add_frame(frame);
}

fn apply_place_object(frame: &mut Frame, place_object: &swf::PlaceObject) {
    if let Some(matrix) = place_object.matrix {
        frame.matrix = Matrix {
            a: matrix.a.to_f32(),
            b: matrix.b.to_f32(),
            c: matrix.c.to_f32(),
            d: matrix.d.to_f32(),
            tx: matrix.tx.to_pixels() as f32,
            ty: matrix.ty.to_pixels() as f32,
        };
    }

    if let Some(color_transform) = place_object.color_transform {
        frame.color_transform = ColorTransform {
            mult_color: color_transform.mult_rgba_normalized(),
            add_color: color_transform.add_rgba_normalized(),
        };
    }

    if let Some(blend_mode) = place_object.blend_mode {
        frame.blend_mode = BlendMode::from_swf_blend_mode(blend_mode);
    }

    // if let Some(filters) = place_object.filters {
    //     frame.filters = filters
    //         .iter()
    //         .map(|filter| Filter::from_swf_filter(filter))
    //         .collect();
    // }
}

fn replace_at_depth(
    time_line: &mut TimeLine,
    place_object: &Box<swf::PlaceObject>,
    id: CharacterId,
    current_frame: u16,
) {
    let mut frame = Frame::new(id, current_frame);
    apply_place_object(&mut frame, &place_object);
    time_line.add_frame(frame);
}

fn remove_at_depth(time_line: &mut TimeLine) {
    // 设置为空白帧 （即不显示任何内容）默认Id为0是空白帧
    time_line.add_frame(Frame::new(CharacterId::default(), 0));
}

fn clear_blank_frame(vector_animation: &mut VectorAnimation) {
    let clear = |animation: &mut Animation| {
        animation
            .time_lines()
            .iter_mut()
            .for_each(|(_, time_line)| time_line.frames.retain(|frame| frame.id != 0));
    };
    vector_animation.animations.values_mut().for_each(clear);

    vector_animation
        .base_animations
        .values_mut()
        .for_each(clear)
}

fn modify_at_depth(
    animation: &mut Animation,
    place_object: &Box<swf::PlaceObject>,
    current_frame: u16,
) {
    let timeline = animation.insert_or_get_time_line(place_object.depth);
    let mut frame = Frame::new(
        timeline.frames.get(timeline.frames.len() - 1).unwrap().id,
        current_frame,
    );
    apply_place_object(&mut frame, &place_object);
    timeline.add_frame(frame);
}
