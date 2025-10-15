use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap, mem};

use swf::{CharacterId, Encoding, Tag};

use crate::{render::filter::Filter, shape::Offset};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FlashAnimation {
    meta: Meta,
    animations: BTreeMap<String, FrameRange>,
    events: BTreeMap<u16, String>,
    root: Vec<Vec<Command>>,
    children: BTreeMap<CharacterId, Mc>,
    shape_offset: BTreeMap<CharacterId, Offset>,
}
impl FlashAnimation {
    fn new(to_string: String, frame_rate: u16) -> Self {
        Self {
            meta: Meta {
                name: to_string,
                frame_rate,
                version: VERSION.to_string(),
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Meta {
    name: String,
    frame_rate: u16,
    version: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FrameRange {
    start_frame: u16,
    end_frame: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Mc {
    Skin {
        name: String,
        skins: BTreeMap<String, CharacterId>,
        commands: Vec<Vec<Command>>,
    },
    Commands(Vec<Vec<Command>>),
}

impl Default for Mc {
    fn default() -> Self {
        Self::Commands(Vec::new())
    }
}

pub fn parse_animations(
    tags: Vec<Tag>,
    shape_offset: BTreeMap<CharacterId, Offset>,
    file_name: &str,
    frame_rate: u16,
    total_frame: u16,
    encoding_for_version: &'static Encoding,
) -> anyhow::Result<FlashAnimation> {
    let mut flash_animation = FlashAnimation::new(file_name.to_string(), frame_rate);
    let mut symbol_classes = BTreeMap::new();
    convert_root_place_object(
        &mut flash_animation,
        &mut symbol_classes,
        &tags,
        encoding_for_version,
    );
    flash_animation.shape_offset = shape_offset;
    calc_animation_frame_range(&mut flash_animation.animations, total_frame);
    convert_child_place_object(
        &mut flash_animation.children,
        &mut symbol_classes,
        tags,
        encoding_for_version,
    );

    Ok(flash_animation)
}

fn calc_animation_frame_range(animations: &mut BTreeMap<String, FrameRange>, total_frame: u16) {
    let mut frame_range = animations.values_mut().collect::<Vec<_>>();
    frame_range.sort_by_key(|e| e.start_frame);
    for i in 0..frame_range.len() - 1 {
        frame_range[i].end_frame = frame_range[i + 1].start_frame - 1;
    }
    frame_range.last_mut().unwrap().end_frame = total_frame;
}

fn convert_root_place_object(
    flash_animation: &mut FlashAnimation,
    symbol_classes: &mut BTreeMap<CharacterId, String>,
    tags: &Vec<Tag>,
    encoding_for_version: &'static Encoding,
) {
    let animations = &mut flash_animation.animations;
    let commands = &mut flash_animation.root;
    let events = &mut flash_animation.events;

    let mut animation_name = Cow::from("Default");
    let mut current_frame = 1;
    let mut frame = Vec::new();
    for tag in tags {
        match tag {
            Tag::PlaceObject(place_object) => {
                if &animation_name == "Default" {
                    animations.insert(
                        animation_name.to_string(),
                        FrameRange {
                            start_frame: current_frame,
                            end_frame: current_frame,
                        },
                    );
                }
                frame.push(Command::PlaceObject(place_object.into()));
            }
            Tag::RemoveObject(remove_object) => {
                frame.push(Command::RemoveObject(remove_object.into()));
            }
            Tag::ShowFrame => {
                current_frame += 1;
                commands.push(mem::take(&mut frame));
            }
            Tag::FrameLabel(frame_label) => {
                let label = frame_label.label.to_str_lossy(encoding_for_version);
                if let Some(event_name) = label.strip_prefix("event_") {
                    events.insert(current_frame, event_name.to_string());
                } else {
                    if let Some(anim_name) = label.strip_prefix("anim_") {
                        animation_name = Cow::Owned(anim_name.to_string());
                    } else {
                        animation_name = Cow::Owned(label.to_string());
                    }
                    animations.insert(
                        animation_name.to_string(),
                        FrameRange {
                            start_frame: current_frame,
                            end_frame: current_frame,
                        },
                    );
                }
            }
            Tag::SymbolClass(symbol_class) => {
                if let Some(first) = symbol_class.first() {
                    symbol_classes.insert(
                        first.id,
                        first.class_name.to_string_lossy(encoding_for_version),
                    );
                }
            }
            _ => {}
        }
    }
}

fn convert_child_place_object(
    children: &mut BTreeMap<CharacterId, Mc>,
    symbol_classes: &mut BTreeMap<CharacterId, String>,
    tags: Vec<Tag>,
    encoding_for_version: &'static Encoding,
) {
    tags.into_iter()
        .filter_map(|tag| match tag {
            Tag::DefineSprite(sprite) => Some(sprite),
            _ => None,
        })
        .for_each(|sprite| {
            let part_name = if let Some(name) = symbol_classes.get(&sprite.id) {
                name.strip_prefix("Skin")
            } else {
                None
            };
            let mut current_frame = 1;
            let mut commands = Vec::new();
            let mut skins = BTreeMap::new();
            let mut frame = Vec::new();
            for tag in sprite.tags {
                match tag {
                    Tag::PlaceObject(place_object) => {
                        frame.push(Command::PlaceObject((&place_object).into()));
                    }
                    Tag::RemoveObject(remove_object) => {
                        frame.push(Command::RemoveObject((&remove_object).into()));
                    }
                    Tag::ShowFrame => {
                        current_frame += 1;
                        commands.push(mem::take(&mut frame));
                    }
                    Tag::FrameLabel(frame_label) => {
                        let label = frame_label.label.to_str_lossy(encoding_for_version);
                        if let Some(skin_name) = label.strip_prefix("skin_") {
                            skins.insert(skin_name.to_string(), current_frame);
                        }
                    }
                    _ => {}
                }
            }
            // 根据是否有名字来决定使用哪个变体
            let mc = if let Some(part_name) = part_name {
                Mc::Skin {
                    name: part_name.to_string(),
                    skins,
                    commands,
                }
            } else {
                Mc::Commands(commands)
            };
            children.insert(sprite.id, mc);
        });
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    PlaceObject(PlaceObject),
    RemoveObject(RemoveObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceObject {
    depth: u16,
    action: PlaceObjectAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    matrix: Option<Matrix>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ratio: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clip_depth: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color_transform: Option<ColorTransform>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blend_mode: Option<BlendMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<Vec<Filter>>,
}

impl From<&Box<swf::PlaceObject<'_>>> for PlaceObject {
    fn from(value: &Box<swf::PlaceObject<'_>>) -> Self {
        Self {
            depth: value.depth,
            action: PlaceObjectAction::from(value.action),
            matrix: value.matrix.map(Into::into),
            ratio: value.ratio,
            clip_depth: value.clip_depth,
            color_transform: value.color_transform.map(Into::into),
            blend_mode: value.blend_mode.map(BlendMode::from_swf_blend_mode),
            filters: value
                .filters
                .as_ref()
                .map(|filters| filters.iter().map(Into::into).collect()),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
enum PlaceObjectAction {
    Place(CharacterId),
    Modify,
    Replace(CharacterId),
}

impl From<swf::PlaceObjectAction> for PlaceObjectAction {
    fn from(value: swf::PlaceObjectAction) -> Self {
        match value {
            swf::PlaceObjectAction::Place(id) => Self::Place(id),
            swf::PlaceObjectAction::Modify => Self::Modify,
            swf::PlaceObjectAction::Replace(id) => Self::Replace(id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveObject {
    pub depth: u16,
    pub character_id: Option<CharacterId>,
}

impl From<&swf::RemoveObject> for RemoveObject {
    fn from(value: &swf::RemoveObject) -> Self {
        Self {
            depth: value.depth,
            character_id: value.character_id,
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Transform {
    matrix: Matrix,
    color_transform: ColorTransform,
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

impl From<swf::Matrix> for Matrix {
    fn from(value: swf::Matrix) -> Self {
        Self {
            a: value.a.to_f32(),
            b: value.b.to_f32(),
            c: value.c.to_f32(),
            d: value.d.to_f32(),
            tx: value.tx.to_pixels() as f32,
            ty: value.ty.to_pixels() as f32,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ColorTransform {
    mult_color: [f32; 4],
    add_color: [f32; 4],
}

impl From<swf::ColorTransform> for ColorTransform {
    fn from(value: swf::ColorTransform) -> Self {
        Self {
            mult_color: value.mult_rgba_normalized(),
            add_color: value.add_rgba_normalized(),
        }
    }
}

impl Default for ColorTransform {
    fn default() -> Self {
        Self {
            mult_color: [1.0, 1.0, 1.0, 1.0],
            add_color: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
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
