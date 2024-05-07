use std::{collections::HashMap, sync::Arc};

use crate::{
    drawing::Drawing,
    tag_utils::{self, ControlFlow, DecodeResult, SwfMovie, SwfSlice, SwfStream},
};
use bitflags::bitflags;
use ruffle_wstr::WString;
use swf::{avm1::read, extensions::ReadSwfExt, read::Reader, CharacterId, TagCode};

use super::{container::ChildContainer, interactive::InteractiveObjectBase};

type FrameNumber = u16;

#[derive(Debug, Clone)]
pub struct MovieClip {
    base: InteractiveObjectBase,

    static_data: MovieClipStatic,
    tag_stream_pos: u64,

    container: ChildContainer,
    flags: MovieClipFlags,

    drawing: Drawing,

    #[cfg(feature = "timeline_debug")]
    tag_frame_boundaries: HashMap<FrameNumber, (u64, u64)>,
}
impl MovieClip {
    pub fn new(movie: Arc<SwfMovie>) -> Self {
        Self {
            base: Default::default(),
            static_data: MovieClipStatic::empty(movie.clone()),
            tag_stream_pos: 0,
            container: ChildContainer::new(movie),
            flags: MovieClipFlags::empty(),
            drawing: Drawing::new(),
            #[cfg(feature = "timeline_debug")]
            tag_frame_boundaries: Default::default(),
        }
    }
    pub fn load(&mut self) {
        let swf = self.static_data.swf.clone();
        let mut reader = Reader::new(swf.data(), swf.version());
        loop {
            let (tag_code, tag_len) = reader.read_tag_code_and_length().unwrap();
            let mut tag_reader = Reader::new(
                reader.read_slice(tag_len).unwrap(),
                self.static_data.swf.version(),
            );
            let tag_code = TagCode::from_u16(tag_code).unwrap();
            let _ = match tag_code {
                TagCode::CsmTextSettings => dbg!("CsmTextSettings"),
                TagCode::DefineBits => dbg!("DefineBits"),
                TagCode::DefineBitsJpeg2 => dbg!("DefineBitsJpeg2"),
                TagCode::DefineBitsJpeg3 => dbg!("DefineBitsJpeg3"),
                TagCode::DefineBitsJpeg4 => dbg!("DefineBitsJpeg4"),
                TagCode::DefineBitsLossless => dbg!("DefineBitsLossless"),
                TagCode::DefineBitsLossless2 => dbg!("DefineBitsLossless2"),
                TagCode::DefineButton => dbg!("DefineButton"),
                TagCode::DefineButton2 => dbg!("DefineButton2"),
                TagCode::DefineButtonCxform => dbg!("DefineButtonCxform"),
                TagCode::DefineButtonSound => dbg!("DefineButtonSound"),
                TagCode::DefineEditText => dbg!("DefineEditText"),
                TagCode::DefineFont => dbg!("DefineFont"),
                TagCode::DefineFont2 => dbg!("DefineFont2"),
                TagCode::DefineFont3 => dbg!("DefineFont3"),
                TagCode::DefineFont4 => dbg!("DefineFont4"),
                TagCode::DefineMorphShape => dbg!("DefineMorphShape"),
                TagCode::DefineMorphShape2 => dbg!("DefineMorphShape2"),
                TagCode::DefineScalingGrid => dbg!("DefineScalingGrid"),
                TagCode::DefineShape => dbg!("DefineShape"),
                TagCode::DefineShape2 => dbg!("DefineShape2"),
                TagCode::DefineShape3 => dbg!("DefineShape3"),
                TagCode::DefineShape4 => dbg!("DefineShape4"),
                TagCode::DefineSound => dbg!("DefineSound"),
                TagCode::DefineVideoStream => dbg!("DefineVideoStream"),
                TagCode::DefineSprite => {
                    self.define_sprite(&mut tag_reader, tag_len);
                    dbg!("DefineSprite")
                }
                TagCode::DefineText => dbg!("DefineText"),
                TagCode::DefineText2 => dbg!("DefineText2"),
                TagCode::DoInitAction => dbg!("DoInitAction"),
                TagCode::DefineSceneAndFrameLabelData => dbg!("DefineSceneAndFrameLabelData"),
                TagCode::ExportAssets => dbg!("ExportAssets"),
                TagCode::FrameLabel => dbg!("FrameLabel"),
                TagCode::JpegTables => dbg!("JpegTables"),
                TagCode::ShowFrame => dbg!("ShowFrame"),
                TagCode::ScriptLimits => dbg!("ScriptLimits"),
                TagCode::SoundStreamHead => dbg!("SoundStreamHead"),
                TagCode::SoundStreamHead2 => dbg!("SoundStreamHead2"),
                TagCode::VideoFrame => dbg!("VideoFrame"),
                TagCode::DefineBinaryData => dbg!("DefineBinaryData"),
                TagCode::ImportAssets => dbg!("ImportAssets"),
                TagCode::ImportAssets2 => dbg!("ImportAssets2"),
                TagCode::End => {
                    dbg!("End");
                    break;
                }
                _ => {
                    dbg!("{}", tag_code);
                    dbg!("Unknown")
                }
            };
        }

        // tag_utils::decode_tags2(reader, tag_callback).unwrap()
    }

    fn define_sprite<'a>(&mut self, reader: &mut SwfStream<'a>, _tag_len: usize) {
        dbg!("DefineSprite");
        let _id = reader.read_character_id().unwrap();
        let _num_frames = reader.read_u16().unwrap();
        self.load();
    }

    pub fn csm_text_settings(&mut self, _reader: &mut Reader) {
        // let settings = reader.read_csm_text_settings()?;
    }
}

bitflags! {
    /// Boolean state flags used by `MovieClip`.
    #[derive(Debug,Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct MovieClipStatic {
    id: CharacterId,
    swf: SwfSlice,

    frame_labels: Vec<(FrameNumber, WString)>,
    frame_labels_map: HashMap<WString, FrameNumber>,
    scene_labels: Vec<Scene>,
    scene_labels_map: HashMap<WString, Scene>,
    total_frames: FrameNumber,
}

impl MovieClipStatic {
    fn empty(movie: Arc<SwfMovie>) -> Self {
        let mcs = Self::with_data(0, SwfSlice::empty(movie), 1);

        mcs
    }
    fn with_data(id: CharacterId, swf: SwfSlice, total_frames: FrameNumber) -> Self {
        Self {
            id,
            swf,
            frame_labels: Vec::new(),
            frame_labels_map: HashMap::new(),
            scene_labels: Vec::new(),
            scene_labels_map: HashMap::new(),
            total_frames,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Scene {
    pub name: WString,
    pub start: FrameNumber,
    pub length: FrameNumber,
}

impl Default for Scene {
    fn default() -> Self {
        Scene {
            name: WString::default(),
            start: 1,
            length: u16::MAX,
        }
    }
}
