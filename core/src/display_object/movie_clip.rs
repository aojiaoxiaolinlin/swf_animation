use std::collections::HashMap;

use crate::{drawing::Drawing, tag_utils::SwfSlice};
use bitflags::bitflags;
use ruffle_wstr::WString;
use swf::{read::Reader, CharacterId, TagCode};

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
    pub fn load(self){
        let data = self.static_data.swf.clone();
        let reader = Reader::new(data.movie.data(),data.movie.version());

        // let tag_callback = |reader: &mut Reader,tag_code,tag_len:usize|{
        //     match tag_code {
        //         // TagCode::CsmTextSettings => self.csm_text_settings(reader),
        //         _ => Ok(()),
        //     }
        // };
    }
    // pub fn csm_text_settings(&mut self, reader: &mut Reader) -> crate::Result<(),> {
    //     let settings = reader.read_csm_text_settings()?;
        
    // }
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
