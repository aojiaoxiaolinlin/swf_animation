use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use swf::{CharacterId, Depth, PlaceObject, PlaceObjectAction, Tag};

use crate::{character::Character, context::UpdateContext, library};

use super::{
    container::TDisplayObjectContainer, graphic::Graphic, DisplayObject, DisplayObjectBase,
    TDisplayObject,
};

type FrameNumber = u16;
#[derive(Clone)]
pub struct MovieClip {
    id: CharacterId,
    pub total_frames: FrameNumber,
    current_frame: FrameNumber,
    base: DisplayObjectBase,
}

impl MovieClip {
    pub fn empty() -> Self {
        Self {
            id: 0,
            total_frames: 1,
            current_frame: 0,
            base: DisplayObjectBase::default(),
        }
    }
    pub fn new_witch_data(id: CharacterId, total_frames: FrameNumber) -> Self {
        Self {
            id,
            current_frame: 0,
            total_frames,
            base: DisplayObjectBase::default(),
        }
    }
    fn instantiate_child(
        &mut self,
        update_context: &mut UpdateContext<'_>,
        id: CharacterId,
        depth: Depth,
        place_object: &Box<PlaceObject>,
    ) {
        let library = update_context.library_mut();
        match library.instantiate_by_id(id) {
            Ok(mut child) => {
                let prev_child = self.replace_at_depth(update_context, &mut child, depth);
                {
                    child.set_instantiated_by_timeline(true);
                    child.set_depth(depth);
                    child.set_place_frame(self.current_frame);
                    
                }
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }
    pub fn run_frame(&mut self, tags: Vec<Tag>, update_context: &mut UpdateContext<'_>) {
        for tag in tags {
            match tag {
                Tag::PlaceObject(place_object) => {
                    match place_object.action {
                        PlaceObjectAction::Place(id) => {
                            self.instantiate_child(
                                update_context,
                                id,
                                place_object.depth.into(),
                                &place_object,
                            );
                        }
                        _ => {}
                    }
                    if let Some(name) = place_object.name {
                        println!("{:?}", name);
                    }
                }
                _ => {}
            }
        }
    }
    pub fn parse_tag(&mut self, tags: Vec<Tag>, update_context: &mut UpdateContext<'_>) {
        for tag in tags {
            match tag {
                Tag::SetBackgroundColor(set_background_color) => {
                    println!("{:?}", set_background_color);
                }
                Tag::DefineSprite(define_sprite) => {
                    let mut movie_clip =
                        MovieClip::new_witch_data(define_sprite.id, define_sprite.num_frames);
                    // let movie_clip = Rc::new(RefCell::new(movie_clip));
                    // 递归解析下一个 MovieClip
                    movie_clip.parse_tag(define_sprite.tags, update_context);
                    // 存入库
                    update_context
                        .library
                        .register_character(define_sprite.id, Character::MovieClip(movie_clip));
                }
                Tag::FrameLabel(frame_label) => {
                    println!("{:?}", frame_label.label);
                }
                Tag::ShowFrame => {}
                Tag::DefineShape(define_shape) => {
                    update_context.library.register_character(
                        define_shape.id,
                        Character::Graphic(Graphic::from_swf_tag(define_shape)),
                    );
                }
                Tag::RemoveObject(remove_object) => {
                    if let Some(character_id) = remove_object.character_id {
                        println!("{:?}", character_id);
                    }
                }
                Tag::DefineSceneAndFrameLabelData(_define_scene_and_frame_label_data) => {}
                // 空
                Tag::DefineBits { id, .. } => {
                    dbg!(id);
                }
                Tag::DefineBitsJpeg2 { id, .. } => {
                    dbg!(id);
                }
                Tag::DefineScalingGrid { id, .. } => {
                    dbg!(id);
                }
                Tag::JpegTables(jpeg_tables) => {
                    dbg!(jpeg_tables.len());
                }
                Tag::DefineMorphShape(define_morph_shape) => {
                    dbg!(define_morph_shape.id);
                }
                Tag::DefineBinaryData(define_binary_data) => {
                    dbg!(define_binary_data.id);
                }
                Tag::DefineBitsLossless(define_bits_lossless) => {
                    dbg!(define_bits_lossless.id);
                }
                Tag::DefineText(_define_text) => {}

                Tag::DefineFont(_define_font) => {}
                Tag::DefineFont2(_define_font2) => {}
                Tag::DefineFont4(_define_font4) => {}
                Tag::DefineFontAlignZones {
                    id: _,
                    thickness: _,
                    zones: _,
                } => {}
                Tag::DefineFontName {
                    id: _,
                    name: _,
                    copyright_info: _,
                } => {}
                _ => {}
            }
        }
    }
}

impl TDisplayObject for MovieClip {
    fn base_mut(&mut self) -> &mut DisplayObjectBase {
        &mut self.base
    }

    fn set_place_frame(&mut self, place_frame: u16) {
        self.base_mut().set_place_frame(place_frame);
    }
}
impl TDisplayObjectContainer for MovieClip {}
