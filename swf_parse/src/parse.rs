use swf::Tag;

use crate::{
    character::Character,
    context::UpdateContext,
    display_object::{graphic::Graphic, movie_clip::MovieClip},
};

pub fn parse_tags(tags: Vec<Tag>, update_context: &mut UpdateContext<'_>) {
    for tag in tags {
        match tag {
            Tag::PlaceObject(place_object) => {
                if let Some(name) = place_object.name {
                    println!("{:?}", name);
                }
            }
            Tag::SetBackgroundColor(set_background_color) => {
                println!("{:?}", set_background_color);
            }
            Tag::DefineSprite(define_sprite) => {
                update_context.library.register_character(
                    define_sprite.id,
                    Character::MovieClip(MovieClip::new(define_sprite.num_frames)),
                );
                parse_tags(define_sprite.tags, update_context);
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
            Tag::RemoveObject(remove_object) => {
                if let Some(character_id) = remove_object.character_id {
                    println!("{:?}", character_id);
                }
            }
            Tag::DefineBitsLossless(define_bits_lossless) => {
                dbg!(define_bits_lossless.id);
            }
            _ => {}
        }
    }
}
