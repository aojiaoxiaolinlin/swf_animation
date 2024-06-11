use crate::{
    character::Character,
    container::{ChildContainer, DisplayObjectContainer, TDisplayObjectContainer},
    context::{self, RenderContext},
    display_object::{DisplayObject, DisplayObjectBase, TDisplayObject},
    drawing::Drawing,
    library::MovieLibrary,
};
use anyhow::anyhow;
use bitflags::bitflags;
use ruffle_render::{
    blend::ExtendedBlendMode,
    commands::{CommandHandler, RenderBlendMode},
};
use swf::{CharacterId, Depth, HeaderExt, PlaceObject, PlaceObjectAction, SwfStr, Tag};

use super::{graphic::Graphic, render_base};

type FrameNumber = u16;
type SwfVersion = u8;
bitflags! {
    /// Boolean state flags used by `MovieClip`.
    #[derive(Clone, Copy)]
    struct MovieClipFlags: u8 {
        /// Whether this `MovieClip` has run its initial frame.
        const INITIALIZED             = 1 << 0;

        /// Whether this `MovieClip` is playing or stopped.
        const PLAYING                 = 1 << 1;

        /// Whether this `MovieClip` has been played as a result of an AS3 command.
        ///
        /// The AS3 `isPlaying` property is broken and yields false until you first
        /// call `play` to unbreak it. This flag tracks that bug.
        const PROGRAMMATICALLY_PLAYED = 1 << 2;

        /// Executing an AVM2 frame script.
        ///
        /// This causes any goto action to be queued and executed at the end of the script.
        const EXECUTING_AVM2_FRAME_SCRIPT = 1 << 3;

        /// Flag set when AVM2 loops to the next frame.
        ///
        /// Because AVM2 queues PlaceObject tags to run later, explicit gotos
        /// that happen while those tags run should cancel the loop.
        const LOOP_QUEUED = 1 << 4;

        const RUNNING_CONSTRUCT_FRAME = 1 << 5;

        /// Whether this `MovieClip` has been post-instantiated yet.
        const POST_INSTANTIATED = 1 << 5;
    }
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
    flags: MovieClipFlags,
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
            flags: MovieClipFlags::empty(),
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
            flags: MovieClipFlags::empty(),
            drawing: Drawing::new(),
        }
    }
    pub fn load_swf(&mut self, tags: Vec<Tag>, library: &mut MovieLibrary) {
        self.parse_tag(tags, library);
    }
    pub fn run_frame_internal(&mut self, tags: Vec<Tag>, library: &mut MovieLibrary) {
        for tag in tags {
            match tag {
                Tag::PlaceObject(place_object) => {
                    self.place_object(place_object, library);
                }
                _ => {}
            }
        }
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

    pub fn render(&mut self, render_context: &mut RenderContext<'_>) {
        render_base(self.clone().into(), render_context);
    }
    fn playing(&self) -> bool {
        self.flags.contains(MovieClipFlags::PLAYING)
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
    fn enter_frame(&mut self) {
        let skip_frame = self.base().should_skip_next_enter_frame();
        for mut child in self.clone().iter_render_list().rev() {
            if skip_frame {
                child.base_mut().set_skip_next_enter_frame(true);
            }
            child.enter_frame();
        }
        if skip_frame {
            self.base_mut().set_skip_next_enter_frame(false);
            return;
        }
        let is_playing = self.playing();
        
        if is_playing {
// self.run_frame_internal(, library)
        }
        
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
