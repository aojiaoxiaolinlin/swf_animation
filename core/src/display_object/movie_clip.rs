use std::sync::Arc;

use crate::{
    character::Character,
    container::{ChildContainer, DisplayObjectContainer, TDisplayObjectContainer},
    context::{self, RenderContext},
    display_object::{DisplayObject, DisplayObjectBase, TDisplayObject},
    drawing::Drawing,
    library::MovieLibrary,
    tag_utils::{self, ControlFlow, Error, SwfMovie, SwfSlice, SwfStream},
};
use anyhow::anyhow;
use bitflags::bitflags;
use ruffle_render::{
    blend::ExtendedBlendMode,
    commands::{CommandHandler, RenderBlendMode},
};
use swf::{
    extensions::ReadSwfExt, read::Reader, CharacterId, Depth, PlaceObject, PlaceObjectAction,
    SwfStr, Tag, TagCode,
};

use super::{graphic::Graphic, render_base};

type FrameNumber = u16;
type SwfVersion = u8;
/// Indication of what frame `run_frame` should jump to next.
#[derive(PartialEq, Eq)]
enum NextFrame {
    /// Construct and run the next frame in the clip.
    Next,

    /// Jump to the first frame in the clip.
    First,

    /// Do not construct or run any frames.
    Same,
}
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
    swf: SwfSlice,
    pub id: CharacterId,
    current_frame: FrameNumber,
    pub total_frames: FrameNumber,
    frame_labels: Vec<(FrameNumber, String)>,
    container: ChildContainer,
    flags: MovieClipFlags,
    tag_stream_pos: u64,
    drawing: Drawing,
}

impl MovieClip {
    pub fn new(movie: Arc<SwfMovie>) -> Self {
        Self {
            base: DisplayObjectBase::default(),
            id: Default::default(),
            current_frame: Default::default(),
            total_frames: movie.num_frames(),
            frame_labels: Default::default(),
            swf: SwfSlice::empty(movie),
            container: ChildContainer::new(),
            flags: MovieClipFlags::empty(),
            drawing: Drawing::new(),
            tag_stream_pos: 0,
        }
    }
    pub fn new_with_data(id: CharacterId, total_frames: FrameNumber, swf: SwfSlice) -> Self {
        Self {
            base: DisplayObjectBase::default(),
            id,
            total_frames,
            current_frame: Default::default(),
            frame_labels: Default::default(),
            swf,
            container: ChildContainer::new(),
            flags: MovieClipFlags::empty(),
            tag_stream_pos: 0,
            drawing: Drawing::new(),
        }
    }
    pub fn swf_version(&self) -> SwfVersion {
        self.swf.version()
    }
    pub fn load_swf(&mut self, library: &mut MovieLibrary) {
        let swf = self.swf.clone();
        let mut reader = Reader::new(&swf.data()[..], swf.version());
        let tag_callback = |reader: &mut SwfStream<'_>, tag_code, tag_len| {
            match tag_code {
                TagCode::SetBackgroundColor => self.set_background_color(library, reader),
                TagCode::DefineShape => self.define_shape(library, reader, 1),
                TagCode::DefineShape2 => self.define_shape(library, reader, 2),
                TagCode::DefineShape3 => self.define_shape(library, reader, 3),
                TagCode::DefineShape4 => self.define_shape(library, reader, 4),
                TagCode::DefineSprite => return self.define_sprite(library, reader, tag_len),
                TagCode::FrameLabel => self.frame_label(reader),
                TagCode::ShowFrame => self.show_frame(reader),
                _ => {
                    println!("tag_code = {:?}", tag_code);
                    Ok(())
                },
            }?;
            Ok(ControlFlow::Continue)
        };

        let _ = tag_utils::decode_tags(&mut reader, tag_callback);
    }
    fn frame_label(&mut self, reader: &mut SwfStream) -> Result<(), Error> {
        let frame_label = reader.read_frame_label()?;
        let label = frame_label
            .label
            .to_str_lossy(SwfStr::encoding_for_version(self.swf.version()));
        self.frame_labels
            .push((self.current_frame, label.into_owned()));
        Ok(())
    }
    fn show_frame(&mut self, reader: &mut SwfStream) -> Result<(), Error> {
        self.current_frame += 1;
        Ok(())
    }
    fn define_sprite(
        &mut self,
        library: &mut MovieLibrary,
        reader: &mut SwfStream,
        tag_len: usize,
    ) -> Result<ControlFlow, Error> {
        let start = reader.as_slice();
        let id = reader.read_character_id()?;
        let num_frames = reader.read_u16()?;
        let num_read = reader.pos(start);

        let mut movice_clip = MovieClip::new_with_data(
            id,
            num_frames,
            self.swf.resize_to_reader(reader, tag_len - num_read),
        );
        movice_clip.load_swf(library);
        library.register_character(id, Character::MovieClip(movice_clip));
        Ok(ControlFlow::Exit)
    }
    fn define_shape(
        &mut self,
        library: &mut MovieLibrary,
        reader: &mut SwfStream,
        version: u8,
    ) -> Result<(), Error> {
        let swf_shape = reader.read_define_shape(version)?;
        let id = swf_shape.id;
        let graphic = Graphic::from_swf_tag(swf_shape);
        library.register_character(id, Character::Graphic(graphic));
        Ok(())
    }
    pub fn run_frame_internal(&mut self, library: &mut MovieLibrary) {
        let next_frame = self.determine_next_frame();
        // match next_frame {
        //     NextFrame::Next => {
        //         return;
        //     }
        //     NextFrame::First => {
        //         todo!()
        //     }
        //     NextFrame::Same => {
        //         todo!()
        //     }
        // }
        let data = self.swf.clone();
        let mut reader = data.read_from(self.tag_stream_pos);
        let tag_callback = |reader: &mut SwfStream<'_>, tag_code, tag_len| {
            match tag_code {
                TagCode::PlaceObject => self.place_object(library, reader, 1),
                TagCode::PlaceObject2 => self.place_object(library, reader, 2),
                TagCode::PlaceObject3 => self.place_object(library, reader, 3),
                TagCode::PlaceObject4 => self.place_object(library, reader, 4),
                TagCode::SetBackgroundColor => self.set_background_color(library, reader),
                TagCode::ShowFrame => return Ok(ControlFlow::Exit),
                _ => Ok(()),
            }?;
            Ok(ControlFlow::Continue)
        };
        let _ = tag_utils::decode_tags(&mut reader, tag_callback);
        let tag_stream_start = self.swf.as_ref().as_ptr() as u64;
        self.tag_stream_pos = reader.get_ref().as_ptr() as u64 - tag_stream_start;
    }
    pub fn set_background_color(
        &mut self,
        library: &mut MovieLibrary,
        reader: &mut SwfStream,
    ) -> Result<(), Error> {
        let background_color = reader.read_rgb()?;
        Ok(())
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

    fn place_object(
        &mut self,
        library: &mut MovieLibrary,
        reader: &mut SwfStream,
        version: SwfVersion,
    ) -> Result<(), Error> {
        let place_object = if version == 1 {
            reader.read_place_object()
        } else {
            reader.read_place_object_2_or_3(version)
        }?;
        match place_object.action {
            PlaceObjectAction::Place(id) => {
                let child = self.instantiate_child(id, place_object.depth, &place_object, library);
                match child {
                    Ok(mut child) => {
                        child.set_depth(place_object.depth);
                        child.set_place_frame(self.current_frame);
                        child.apply_place_object(&place_object, self.swf.version());
                        if let Some(name) = &place_object.name {
                            child.set_name(Some(
                                name.to_str_lossy(SwfStr::encoding_for_version(self.swf.version()))
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
                    child.apply_place_object(&place_object, self.swf.version());
                    child.set_place_frame(self.current_frame);
                }
            }
            PlaceObjectAction::Modify => {
                if let Some(mut child) = self.child_by_depth(place_object.depth.into()) {
                    child.apply_place_object(&place_object, self.swf.version());
                }
            }
        }
        Ok(())
    }

    pub fn render(&mut self, render_context: &mut RenderContext<'_>) {
        render_base(self.clone().into(), render_context);
    }
    fn playing(&self) -> bool {
        self.flags.contains(MovieClipFlags::PLAYING)
    }
    pub fn movie(&self) -> Arc<SwfMovie> {
        self.swf.movie.clone()
    }
    pub fn tag_stream_len(&self) -> usize {
        self.swf.end - self.swf.start
    }
    pub fn total_bytes(self) -> i32 {
        // For a loaded SWF, returns the uncompressed size of the SWF.
        // Otherwise, returns the size of the tag list in the clip's DefineSprite tag.
        if self.is_root() {
            self.movie().uncompressed_len()
        } else {
            self.tag_stream_len() as i32
        }
    }
    fn determine_next_frame(&self) -> NextFrame {
        if self.current_frame < self.total_frames {
            NextFrame::Next
        } else if self.total_frames > 1 {
            NextFrame::First
        } else {
            NextFrame::Same
        }
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
