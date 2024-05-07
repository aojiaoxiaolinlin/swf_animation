use core::str;
use std::{
    collections::{hash_map, HashMap},
    io::Read,
    sync::Arc,
};

use crate::{
    character::{self, Character},
    library,
    tag_utils::{Error, SwfMovie},
};
use crate::{
    context::{self, UpdateContext},
    library::Library,
    string::SwfStrExt,
    tag_utils::SwfSlice,
};
use ruffle_wstr::WString;
use swf::{
    extensions::ReadSwfExt,
    read::{ControlFlow, Reader},
    FrameLabelData, TagCode,
};

use super::{graphic::Graphic, morph_shape::MorphShape};
type FrameNumber = u16;
pub struct MovieClip {
    static_data: MovieClipData,
}

impl MovieClip {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            static_data: MovieClipData {
                swf: SwfSlice::from(Arc::new(SwfMovie::from_data(&data).unwrap())),
                total_frames: 1,
                frame_labels: Vec::new(),
                frame_labels_map: HashMap::new(),
                scene_labels: Vec::new(),
                scene_labels_map: HashMap::new(),
                frame_range: Default::default(),
            },
        }
    }

    pub fn parse(&mut self) {
        let swf_movie = self.static_data.swf.clone().movie.clone();
        let mut reader = Reader::new(&swf_movie.data(), swf_movie.version());
        let mut context = UpdateContext {
            library: Library::empty(),
            player_version: swf_movie.version(),
        };
        self.read(&mut context, &mut reader);
    }

    pub fn read(&mut self, context: &mut UpdateContext, reader: &mut Reader) {
        let tag_callback = |tag_reader: &mut Reader, tag_code: TagCode| -> ControlFlow {
            match tag_code {
                TagCode::End => {
                    println!("End");
                    return ControlFlow::Break;
                }
                TagCode::ShowFrame => {
                    self.show_frame(tag_reader).unwrap();
                }
                TagCode::CsmTextSettings => {
                    self.csm_text_settings(context, tag_reader).unwrap();
                }
                TagCode::DefineBinaryData => {
                    self.define_binary_data(tag_reader).unwrap();
                }
                TagCode::DefineBits => {
                    self.define_bits(tag_reader).unwrap();
                }
                TagCode::DefineBitsJpeg2 => {
                    self.define_bits_jpeg_2(tag_reader).unwrap();
                }
                TagCode::DefineBitsJpeg3 => {
                    self.define_bits_jpeg_3_or_4(tag_reader, 3).unwrap();
                }
                TagCode::DefineBitsJpeg4 => {
                    println!("Define bits jpeg4");
                }
                TagCode::DefineButton => {
                    println!("Define button");
                }
                TagCode::DefineButton2 => {
                    println!("Define button2");
                }
                TagCode::DefineButtonCxform => {
                    println!("Define button cxform");
                }
                TagCode::DefineButtonSound => {
                    println!("Define button sound");
                }
                TagCode::DefineEditText => {
                    println!("Define edit text");
                }
                TagCode::DefineFont => {
                    println!("Define font");
                }
                TagCode::DefineFont2 => {
                    println!("Define font2");
                }
                TagCode::DefineFont3 => {
                    println!("Define font3");
                }
                TagCode::DefineFont4 => {
                    println!("Define font4");
                }
                TagCode::DefineFontAlignZones => {
                    println!("Define font align zones");
                }
                TagCode::DefineFontInfo => {
                    println!("Define font info");
                }
                TagCode::DefineFontInfo2 => {
                    println!("Define font info2");
                }
                TagCode::DefineFontName => {
                    println!("Define font name");
                }
                TagCode::DefineMorphShape => {
                    self.define_morph_shape(context, tag_reader, 1).unwrap();
                }
                TagCode::DefineMorphShape2 => {
                    self.define_morph_shape(context, tag_reader, 2).unwrap();
                }
                TagCode::DefineShape => {
                    self.define_shape(context, tag_reader, 1).unwrap();
                }
                TagCode::DefineShape2 => {
                    self.define_shape(context, tag_reader, 2).unwrap();
                }
                TagCode::DefineShape3 => {
                    self.define_shape(context, tag_reader, 3).unwrap();
                }
                TagCode::DefineShape4 => {
                    self.define_shape(context, tag_reader, 4).unwrap();
                }
                TagCode::DefineSound => {
                    println!("Define sound");
                }
                TagCode::DefineText => {
                    println!("Define text");
                }
                TagCode::DefineText2 => {
                    println!("Define text2");
                }
                TagCode::DefineVideoStream => {
                    self.define_video_stream(tag_reader).unwrap();
                }
                TagCode::EnableTelemetry => {
                    println!("Enable telemetry");
                }
                TagCode::ImportAssets => {
                    println!("Import assets");
                }
                TagCode::ImportAssets2 => {
                    println!("Import assets2");
                }

                TagCode::JpegTables => {
                    self.jpeg_tables(tag_reader).unwrap();
                }

                TagCode::Metadata => {
                    println!("Metadata");
                }

                TagCode::SetBackgroundColor => {
                    println!("Set background color");
                }

                TagCode::SoundStreamBlock => {
                    println!("Sound stream block");
                }
                TagCode::SoundStreamHead => {
                    println!("Sound stream head");
                }

                TagCode::SoundStreamHead2 => {
                    println!("Sound stream head2");
                }
                TagCode::StartSound => {
                    println!("Start sound");
                }

                TagCode::StartSound2 => {
                    println!("Start sound2");
                }
                TagCode::DebugId => {
                    println!("Debug id");
                }
                TagCode::DefineBitsLossless => {
                    self.define_bits_lossless(tag_reader, 1).unwrap();
                }
                TagCode::DefineBitsLossless2 => {
                    self.define_bits_lossless(tag_reader, 2).unwrap();
                }

                TagCode::DefineScalingGrid => {
                    self.define_scaling_grid(context, tag_reader).unwrap();
                }

                TagCode::DoAbc => {
                    println!("Do abc");
                }
                TagCode::DoAbc2 => {
                    println!("Do abc2");
                }

                TagCode::DoAction => {
                    println!("Do action");
                }

                TagCode::DoInitAction => {
                    println!("Do init action");
                }

                TagCode::EnableDebugger => {
                    println!("Enable debugger");
                }
                TagCode::EnableDebugger2 => {
                    println!("Enable debugger2");
                }
                TagCode::ScriptLimits => {
                    println!("Script limits");
                }
                TagCode::SetTabIndex => {
                    println!("Set tab index");
                }
                TagCode::SymbolClass => {
                    println!("Symbol class");
                }

                TagCode::ExportAssets => {
                    self.export_assets(context, tag_reader).unwrap();
                }

                TagCode::FileAttributes => {
                    println!("File attributes");
                }

                TagCode::Protect => {
                    println!("Protect");
                }

                TagCode::DefineSceneAndFrameLabelData => {
                    self.scene_and_frame_labels(context, tag_reader).unwrap();
                }

                TagCode::FrameLabel => {
                    self.frame_label(context, tag_reader).unwrap();
                }

                TagCode::DefineSprite => {
                    println!("Define sprite");
                    self.read_define_sprite(context, tag_reader);
                }
                TagCode::PlaceObject => {
                    println!("Place object");
                }
                TagCode::PlaceObject2 => {
                    println!("Place object2");
                }
                TagCode::PlaceObject3 => {
                    println!("Place object3");
                }
                TagCode::PlaceObject4 => {
                    println!("Place object4");
                }

                TagCode::RemoveObject => {
                    println!("Remove object");
                }

                TagCode::RemoveObject2 => {
                    println!("Remove object2");
                }

                TagCode::VideoFrame => {
                    self.preload_video_frame(tag_reader).unwrap();
                }
                TagCode::ProductInfo => {
                    println!("Product info");
                }
                TagCode::NameCharacter => {
                    println!("Name character");
                }
            };
            ControlFlow::Continue
        };
        reader.read_tag_code(tag_callback);
    }
    #[inline]
    fn read_define_sprite(&mut self, context: &mut UpdateContext, tag_reader: &mut Reader) {
        let _id = tag_reader.read_u16();
        let _num_frames = tag_reader.read_u16();
        self.read(context, tag_reader);
    }
    #[inline]
    fn csm_text_settings(
        &mut self,
        context: &mut UpdateContext,
        reader: &mut Reader,
    ) -> Result<(), Error> {
        let _settings = reader.read_csm_text_settings()?;
        let _library = context.library.library_for_movie_mut(self.movie());

        Ok(())
    }
    #[inline]
    fn define_shape(
        &mut self,
        context: &mut UpdateContext,
        reader: &mut Reader,
        version: u8,
    ) -> Result<(), Error> {
        let swf_shape = reader.read_define_shape(version)?;
        let id = swf_shape.id;
        let graphic = Graphic::from_swf_tag(swf_shape, self.movie());
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(id, Character::Graphic(graphic));
        Ok(())
    }
    #[inline]
    fn preload_video_frame(&mut self, reader: &mut Reader) -> Result<(), Error> {
        let video_frame = reader.read_video_frame()?;
        Ok(())
    }
    #[inline]
    fn define_bits(&mut self, reader: &mut Reader) -> Result<(), Error> {
        let id = reader.read_u16()?;
        let jpeg_data = reader.read_slice_to_end();
        // let jpeg_data = ruffle_render::utils::glue_tables_to_jpeg(jpeg_data, jpeg_tables).into_owned();
        let (width, height) = ruffle_render::utils::decode_define_bits_jpeg_dimensions(&jpeg_data)?;
        dbg!(width, height);
        Ok(())
    }
    #[inline]
    fn define_bits_jpeg_2(&mut self, reader: &mut Reader) -> Result<(), Error> {
        let id = reader.read_u16()?;
        let jpeg_data = reader.read_slice_to_end();
        let (width, height) = ruffle_render::utils::decode_define_bits_jpeg_dimensions(&jpeg_data)?;
        Ok(())
    }
    #[inline]
    fn define_bits_jpeg_3_or_4(&mut self, reader: &mut Reader, version: u8) -> Result<(), Error> {
        let id = reader.read_u16()?;
        let jpeg_len = reader.read_u32()? as usize;
        if version == 4 {
            let _deblocking = reader.read_u16()?;
        }
        let jpeg_data = reader.read_slice(jpeg_len)?;
        let alpha_data = reader.read_slice_to_end();
        let (width, height) = ruffle_render::utils::decode_define_bits_jpeg_dimensions(&jpeg_data)?;
        Ok(())
    }
    #[inline]
    fn define_bits_lossless(&mut self, reader: &mut Reader, version: u8) -> Result<(), Error> {
        let define_bits_lossless = reader.read_define_bits_lossless(version)?;
        Ok(())
    }
    #[inline]
    fn define_morph_shape(
        &mut self,
        context: &mut UpdateContext,
        reader: &mut Reader,
        version: u8,
    ) -> Result<(), Error> {
        let morph_shape = reader.read_define_morph_shape(version)?;
        let id = morph_shape.id;
        let morph_shape = MorphShape::from_swf_tag(morph_shape, self.movie());
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(id, Character::MorphShape(morph_shape));
        Ok(())
    }
    #[inline]
    fn define_scaling_grid(
        &mut self,
        context: &mut UpdateContext,
        reader: &mut Reader,
    ) -> Result<(), Error> {
        let id = reader.read_u16()?;
        let rect = reader.read_rectangle()?;
        let library = context.library.library_for_movie_mut(self.movie());
        if let Some(character) = library.character_by_id(id) {
            if let Character::MovieClip(clip) = character {
                // clip.set_scaling_grid(rect);
            } else {
                println!("Movie clip {}: Scaling grid on non-movie clip", id);
            }
        }
        Ok(())
    }
    #[inline]
    fn define_video_stream(&mut self, reader: &mut Reader) -> Result<(), Error> {
        let video_stream = reader.read_define_video_stream()?;
        let id = video_stream.id;
        Ok(())
    }
    #[inline]
    fn scene_and_frame_labels(
        &mut self,
        context: &mut UpdateContext,
        reader: &mut Reader,
    ) -> Result<(), Error> {
        let static_data = &mut self.static_data;
        let mut sfl_data = reader.read_define_scene_and_frame_label_data()?;
        sfl_data
            .scenes
            .sort_unstable_by(|s1, s2| s1.frame_num.cmp(&s2.frame_num));
        for (i, FrameLabelData { frame_num, label }) in sfl_data.scenes.iter().enumerate() {
            let start = *frame_num as u16 + 1;
            let end = sfl_data
                .scenes
                .get(i + 1)
                .map(|s| s.frame_num as u16)
                .unwrap_or(static_data.total_frames + 1);
            let scene = Scene {
                name: label.decode(reader.encoding()).into_owned(),
                start,
                length: end - start,
            };
            static_data.scene_labels.push(scene.clone());
            if let hash_map::Entry::Vacant(entry) =
                static_data.scene_labels_map.entry(scene.name.clone())
            {
                entry.insert(scene);
            } else {
                // println!("Movie clip {}: Duplicated scene label", self.id());
            }
        }

        Ok(())
    }
    #[inline]
    fn export_assets(
        &mut self,
        context: &mut UpdateContext,
        reader: &mut Reader,
    ) -> Result<(), Error> {
        let exports = reader.read_export_assets()?;
        for export in exports {
            let name = export.name.decode(reader.encoding());
        }
        Ok(())
    }
    #[inline]
    fn jpeg_tables(&mut self, reader: &mut Reader) -> Result<(), Error> {
        let jpeg_data = reader.read_slice_to_end();
        Ok(())
    }
    #[inline]
    fn define_binary_data(&mut self, reader: &mut Reader) -> Result<(), Error> {
        let tag_data = reader.read_define_binary_data()?;
        Ok(())
    }
    #[inline]
    fn show_frame(&mut self, reader: &mut Reader) -> Result<(), Error> {
        let cur_frame = self.static_data.cur_frame();
        self.static_data.frame_range.cur_frame = cur_frame + 1;
        Ok(())
    }
    #[inline]
    fn frame_label(
        &mut self,
        context: &mut UpdateContext,
        reader: &mut Reader,
    ) -> Result<(), Error> {
        let frame_label = reader.read_frame_label()?;
        let mut label = frame_label.label.decode(reader.encoding()).into_owned();
        if !self.movie().is_action_script_3() {
            label.make_ascii_lowercase();
        }
        let static_data = &mut self.static_data;
        if !static_data.scene_labels.is_empty() {
            return Ok(());
        }

        let current_frame = static_data.cur_frame();
        static_data
            .frame_labels
            .push((static_data.cur_frame(), label.clone()));
        if let hash_map::Entry::Vacant(entry) = static_data.frame_labels_map.entry(label) {
            entry.insert(current_frame);
        } else {
            // println!("Movie clip {}: Duplicated frame label", self.id());
        }
        Ok(())
    }

    pub fn movie(&self) -> Arc<SwfMovie> {
        self.static_data.swf.movie.clone()
    }
}

pub struct MovieClipData {
    // swf_movie: Arc<SwfMovie>,
    swf: SwfSlice,
    frame_labels: Vec<(FrameNumber, WString)>,
    frame_labels_map: HashMap<WString, FrameNumber>,
    total_frames: FrameNumber,
    scene_labels: Vec<Scene>,
    scene_labels_map: HashMap<WString, Scene>,
    frame_range: FrameRange,
}
impl MovieClipData {
    fn cur_frame(&self) -> FrameNumber {
        self.frame_range.cur_frame
    }
}
#[derive(Clone)]
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
struct FrameRange {
    cur_frame: FrameNumber,
    last_frame_start_pos: FrameNumber,
}
impl Default for FrameRange {
    fn default() -> Self {
        FrameRange {
            cur_frame: 1,
            last_frame_start_pos: 0,
        }
    }
}
