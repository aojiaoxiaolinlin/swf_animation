use std::{cell::RefCell, rc::Rc};

use swf::{CharacterId, Depth, PlaceObject, PlaceObjectAction, Tag};

use crate::{character::Character, context::UpdateContext, library};

use super::{container::TDisplayObjectContainer, graphic::Graphic, DisplayObjectBase, TDisplayObject};

type FrameNumber = u16;
#[derive(Debug)]
pub struct MovieClip {
    id: CharacterId,
    pub total_frames: u16,
}

impl MovieClip {
    pub fn empty() -> Self {
        Self {
            id: 0,
            total_frames: 1,
        }
    }
    pub fn new_witch_data(id: CharacterId, total_frames: FrameNumber) -> Self {
        Self { id, total_frames }
    }
    fn instantiate_child(
        &mut self,
        update_context: &mut UpdateContext<'_>,
        id: CharacterId,
        depth: Depth,
        place_object: &Box<PlaceObject>,
    ) {
        let library = update_context.library_mut();
        match library.instantiate_by_id(id){
            Ok(child)=>{
                let prev_child = self.replace_at_depth(update_context,child,depth);
                {
                    
                }
            }
            Err(e)=>{
                dbg!(e);
            }
        }
    }
    pub fn parese_tag(&mut self, tags: Vec<Tag>, update_context: &mut UpdateContext<'_>) {
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
                Tag::SetBackgroundColor(set_background_color) => {
                    println!("{:?}", set_background_color);
                }
                Tag::DefineSprite(define_sprite) => {
                    let movie_clip =
                        MovieClip::new_witch_data(define_sprite.id, define_sprite.num_frames);
                    let movie_clip = Rc::new(RefCell::new(movie_clip));
                    // 存入库
                    update_context.library.register_character(
                        define_sprite.id,
                        Character::MovieClip(movie_clip.clone()),
                    );
                    // 递归解析下一个 MovieClip
                    movie_clip
                        .borrow_mut()
                        .parese_tag(define_sprite.tags, update_context);
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
                _ => {}
            }
        }
    }
}


impl TDisplayObject for Rc<RefCell<MovieClip>>{
}

impl TDisplayObjectContainer for MovieClip {
    
}