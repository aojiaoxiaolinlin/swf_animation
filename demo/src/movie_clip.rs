use crate::{
    character::Character,
    container::ChildContainer,
    display_object::{DisplayObjectBase, TDisplayObject},
    graphic::Graphic,
    library::MovieLibrary,
};
use anyhow::anyhow;
use swf::{CharacterId, Depth, HeaderExt, PlaceObjectAction, SwfStr, Tag};

type FrameNumber = u16;
type SwfVersion = u8;

#[derive(Clone)]
pub struct MovieClip {
    base: DisplayObjectBase,
    swf_version: SwfVersion,
    id: CharacterId,
    current_frame: FrameNumber,
    total_frames: FrameNumber,
    frame_labels: Vec<(FrameNumber, String)>,
    container: ChildContainer,
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
        }
    }
    pub fn load_swf(&mut self, tags: Vec<Tag>, library: &mut MovieLibrary) {
        self.parse_tag(tags, library);
    }
    pub fn parse_tag(&mut self, tags: Vec<Tag>, library: &mut MovieLibrary) {
        for tag in tags {
            match tag {
                Tag::PlaceObject(place_object) => match place_object.action {
                    PlaceObjectAction::Place(id) => {
                        let child =
                            self.instantiate_child(id, place_object.depth, &place_object, library);
                        match child {
                            Ok(mut child) => {
                                child.apply_place_object(&place_object, self.swf_version);
                                if let Some(name) = &place_object.name {
                                    child.set_name(Some(
                                        name.to_str_lossy(SwfStr::encoding_for_version(
                                            self.swf_version,
                                        ))
                                        .into_owned(),
                                    ));
                                }
                                if let Some(clip_depth) = place_object.clip_depth {
                                    child.set_clip_depth(clip_depth);
                                }
                            }
                            Err(e) => {
                                dbg!(e);
                            }
                        }
                    }
                    PlaceObjectAction::Replace(id) => {}
                    PlaceObjectAction::Modify => {}
                },
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
                Tag::ShowFrame => {}
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
    ) -> anyhow::Result<Box<dyn TDisplayObject>> {
        if let Some(character) = library.character(id) {
            match character.clone() {
                Character::MovieClip(movie_clip) => Ok(Box::new(movie_clip)),
                Character::Graphic(graphic) => Ok(Box::new(graphic)),
            }
        } else {
            Err(anyhow!("Character id doesn't exist"))
        }
    }
    pub fn frame_labels(&self) -> &[(FrameNumber, String)] {
        &self.frame_labels
    }
}

impl TDisplayObject for MovieClip {
    fn base_mut(&mut self) -> &mut DisplayObjectBase {
        &mut self.base
    }
}
