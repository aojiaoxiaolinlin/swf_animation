use crate::{
    character::Character,
    container::{ChildContainer, DisplayObjectContainer, TDisplayObjectContainer},
    context::{self, RenderContext},
    display_object::{DisplayObject, DisplayObjectBase, TDisplayObject},
    drawing::Drawing,
    library::MovieLibrary,
};
use anyhow::anyhow;
use ruffle_render::{
    blend::ExtendedBlendMode,
    commands::{CommandHandler, RenderBlendMode},
};
use swf::{CharacterId, Depth, HeaderExt, PlaceObject, PlaceObjectAction, SwfStr, Tag};

use super::{graphic::Graphic, render_base};

type FrameNumber = u16;
type SwfVersion = u8;

#[derive(PartialEq, Eq)]
enum NextFrame {
    /// Construct and run the next frame in the clip.
    Next,

    /// Jump to the first frame in the clip.
    First,

    /// Do not construct or run any frames.
    Same,
}
#[derive(Clone)]
pub struct MovieClip {
    base: DisplayObjectBase,
    swf_version: SwfVersion,
    pub id: CharacterId,
    current_frame: FrameNumber,
    pub total_frames: FrameNumber,
    frame_labels: Vec<(FrameNumber, String)>,
    container: ChildContainer,
    drawing: Drawing,
}

impl MovieClip {
    pub fn new(header: HeaderExt) -> Self {
        Self {
            base: DisplayObjectBase::default(),
            id: Default::default(),
            current_frame: Default::default(),
            total_frames: header.num_frames(),
            frame_labels: Default::default(),
            swf_version: header.version(),
            container: ChildContainer::new(),
            drawing: Drawing::new(),
        }
    }
    pub fn new_with_data(
        id: CharacterId,
        total_frames: FrameNumber,
        swf_version: SwfVersion,
    ) -> Self {
        Self {
            base: DisplayObjectBase::default(),
            id,
            total_frames,
            current_frame: Default::default(),
            frame_labels: Default::default(),
            swf_version,
            container: ChildContainer::new(),
            drawing: Drawing::new(),
        }
    }
    pub fn load_swf(&mut self, tags: Vec<Tag>, library: &mut MovieLibrary) {
        self.parse_tag(tags, library);
    }
    pub fn parse_tag(&mut self, tags: Vec<Tag>, library: &mut MovieLibrary) {
        for tag in tags {
            match tag {
                Tag::PlaceObject(place_object) => {
                    self.place_object(place_object, library);
                }
                Tag::SetBackgroundColor(set_background_color) => {
                    println!("{:?}", set_background_color);
                }
                Tag::DefineSprite(define_sprite) => {
                    let mut movie_clip = MovieClip::new_with_data(
                        define_sprite.id,
                        define_sprite.num_frames,
                        self.swf_version,
                    );
                    // let movie_clip = Rc::new(RefCell::new(movie_clip));
                    // 递归解析下一个 MovieClip
                    movie_clip.parse_tag(define_sprite.tags, library);
                    // 存入库
                    library.register_character(define_sprite.id, Character::MovieClip(movie_clip));
                }
                Tag::FrameLabel(frame_label) => {
                    self.frame_labels.push((
                        self.current_frame,
                        frame_label
                            .label
                            .to_str_lossy(SwfStr::encoding_for_version(self.swf_version))
                            .into_owned(),
                    ));
                }
                Tag::ShowFrame => {
                    self.current_frame += 1;
                }
                Tag::DefineShape(define_shape) => {
                    library.register_character(
                        define_shape.id,
                        Character::Graphic(Graphic::from_swf_tag(define_shape)),
                    );
                }
                Tag::RemoveObject(_remove_object) => {
                    // dbg!(remove_object.depth);
                }
                Tag::DefineSceneAndFrameLabelData(_define_scene_and_frame_label_data) => {}
                _ => {}
            }
        }
    }
    fn instantiate_child(
        &mut self,
        id: CharacterId,
        depth: Depth,
        place_object: &swf::PlaceObject,
        library: &mut MovieLibrary,
    ) -> anyhow::Result<DisplayObject> {
        if let Some(character) = library.character(id) {
            match character.clone() {
                Character::MovieClip(movie_clip) => Ok(movie_clip.into()),
                Character::Graphic(graphic) => Ok(graphic.into()),
            }
        } else {
            Err(anyhow!("Character id doesn't exist"))
        }
    }
    fn place_object(&mut self, place_object: Box<PlaceObject>, library: &mut MovieLibrary) {
        match place_object.action {
            PlaceObjectAction::Place(id) => {
                let child = self.instantiate_child(id, place_object.depth, &place_object, library);
                match child {
                    Ok(mut child) => {
                        child.set_depth(place_object.depth);
                        child.set_place_frame(self.current_frame);
                        child.apply_place_object(&place_object, self.swf_version);
                        if let Some(name) = &place_object.name {
                            child.set_name(Some(
                                name.to_str_lossy(SwfStr::encoding_for_version(self.swf_version))
                                    .into_owned(),
                            ));
                        }
                        if let Some(clip_depth) = place_object.clip_depth {
                            child.set_clip_depth(clip_depth);
                        }
                        child.post_instantiation(library);
                        self.replace_at_depth(place_object.depth, child);
                    }
                    Err(_e) => {}
                }
            }
            PlaceObjectAction::Replace(id) => {
                if let Some(mut child) = self.child_by_depth(place_object.depth.into()) {
                    child.replace_with(id, library);
                    child.apply_place_object(&place_object, self.swf_version);
                    child.set_place_frame(self.current_frame);
                }
            }
            PlaceObjectAction::Modify => {
                if let Some(mut child) = self.child_by_depth(place_object.depth.into()) {
                    child.apply_place_object(&place_object, self.swf_version);
                }
            }
        }
    }
    pub fn frame_labels(&self) -> &[(FrameNumber, String)] {
        &self.frame_labels
    }
    fn determine_next_frame(self) -> NextFrame {
        if self.current_frame < self.total_frames {
            NextFrame::Next
        } else if self.total_frames > 1 {
            NextFrame::First
        } else {
            NextFrame::Same
        }
    }
    pub fn render(&mut self, render_context: &mut RenderContext<'_>) {
        render_base(self.clone().into(), render_context);
    }
}

impl TDisplayObject for MovieClip {
    fn base_mut(&mut self) -> &mut DisplayObjectBase {
        &mut self.base
    }

    fn base(&self) -> &DisplayObjectBase {
        &self.base
    }

    fn character_id(&self) -> CharacterId {
        self.id
    }

    fn as_children(self) -> Option<DisplayObjectContainer> {
        Some(self.into())
    }
    fn as_movie(&mut self) -> Option<MovieClip> {
        Some(self.clone())
    }

    fn render_self(&self, render_context: &mut RenderContext<'_>) {
        self.drawing.render(render_context);
        self.render_children(render_context);
    }

    fn self_bounds(&self) -> swf::Rectangle<swf::Twips> {
        self.drawing.self_bounds().clone()
    }
}
impl From<MovieClip> for DisplayObject {
    fn from(movie_clip: MovieClip) -> Self {
        DisplayObject::MovieClip(movie_clip)
    }
}
impl TDisplayObjectContainer for MovieClip {
    fn raw_container(&self) -> &ChildContainer {
        &self.container
    }

    fn raw_container_mut(&mut self) -> &mut ChildContainer {
        &mut self.container
    }
}

impl From<MovieClip> for DisplayObjectContainer {
    fn from(movie_clip: MovieClip) -> Self {
        DisplayObjectContainer::MovieClip(movie_clip)
    }
}
