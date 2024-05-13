use std::{path::Path, sync::Arc};

use swf::{CharacterId, Fixed8, HeaderExt, Rectangle, Twips};
use thiserror::Error;
pub type SwfStream<'a> = swf::read::Reader<'a>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't read SWF: {0}")]
    InvalidSwf(#[from] swf::error::Error),

    #[error("Couldn't register bitmap: {0}")]
    InvalidBitmap(#[from] ruffle_render::error::Error),

    // #[error("Couldn't register font: {0}")]
    // InvalidFont(#[from] ttf_parser::FaceParsingError),
    #[error("Attempted to set symbol classes on movie without any")]
    NoSymbolClasses,

    #[error("Attempted to preload video frames into non-video character {0}")]
    PreloadVideoIntoInvalidCharacter(CharacterId),

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Invalid SWF url")]
    InvalidSwfUrl,
}
#[derive(Debug, Clone)]
pub struct SwfMovie {
    /// The SWF header parsed from the data stream.
    header: HeaderExt,

    /// Uncompressed SWF data.
    data: Vec<u8>,

    /// The suggest encoding for this SWF.
    encoding: &'static swf::Encoding,

    /// The compressed length of the entire datastream
    compressed_len: usize,

    /// Whether this SwfMovie actually represents a loaded movie or fills in for
    /// something else, like an loaded image, filler movie, or error state.
    is_movie: bool,
}

impl SwfMovie {
    pub fn empty(swf_version: u8) -> Self {
        Self {
            header: HeaderExt::default_with_swf_version(swf_version),
            data: vec![],
            encoding: swf::UTF_8,
            compressed_len: 0,
            is_movie: false,
        }
    }
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let abs_path = path.as_ref().canonicalize().unwrap();
        let swf_data = std::fs::read(abs_path)?;
        Self::from_data(&swf_data)
    }
    pub fn from_data(swf_data: &[u8]) -> Result<Self, Error> {
        let compressed_len = swf_data.len();
        let swf_buf = swf::read::decompress_swf(swf_data)?;
        let encoding = swf::SwfStr::encoding_for_version(swf_buf.header.version());
        let movie = Self {
            header: swf_buf.header,
            data: swf_buf.data,
            encoding,
            compressed_len,
            is_movie: true,
        };
        Ok(movie)
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
    pub fn stage_size(&self) -> &Rectangle<Twips> {
        self.header.stage_size()
    }
    pub fn width(&self) -> Twips {
        self.stage_size().width()
    }
    pub fn height(&self) -> Twips {
        self.stage_size().height()
    }
    pub fn num_frames(&self) -> u16 {
        self.header.num_frames()
    }
    pub fn frame_rate(&self) -> Fixed8 {
        self.header.frame_rate()
    }
    pub fn version(&self) -> u8 {
        self.header.version()
    }
    pub fn header(&self) -> &HeaderExt {
        &self.header
    }
    pub fn is_action_script_3(&self) -> bool {
        self.header.is_action_script_3()
    }
}

/// A shared-ownership reference to some portion of an SWF datastream.
#[derive(Debug, Clone)]
pub struct SwfSlice {
    pub movie: Arc<SwfMovie>,
    pub start: usize,
    pub end: usize,
}

impl From<Arc<SwfMovie>> for SwfSlice {
    fn from(movie: Arc<SwfMovie>) -> Self {
        let end = movie.data().len();

        Self {
            movie,
            start: 0,
            end,
        }
    }
}

impl AsRef<[u8]> for SwfSlice {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.data()
    }
}

impl SwfSlice {
    /// Creates an empty SwfSlice.
    #[inline]
    pub fn empty(movie: Arc<SwfMovie>) -> Self {
        Self {
            movie,
            start: 0,
            end: 0,
        }
    }

    /// Creates an empty SwfSlice of the same movie.
    #[inline]
    pub fn copy_empty(&self) -> Self {
        Self::empty(self.movie.clone())
    }

    /// Construct a new SwfSlice from a regular slice.
    ///
    /// This function returns None if the given slice is not a subslice of the
    /// current slice.
    pub fn to_subslice(&self, slice: &[u8]) -> Self {
        let self_pval = self.movie.data().as_ptr() as usize;
        let slice_pval = slice.as_ptr() as usize;

        if (self_pval + self.start) <= slice_pval && slice_pval < (self_pval + self.end) {
            Self {
                movie: self.movie.clone(),
                start: slice_pval - self_pval,
                end: (slice_pval - self_pval) + slice.len(),
            }
        } else {
            self.copy_empty()
        }
    }

    /// Construct a new SwfSlice from a movie subslice.
    ///
    /// This function allows subslices outside the current slice to be formed,
    /// as long as they are valid subslices of the movie itself.
    pub fn to_unbounded_subslice(&self, slice: &[u8]) -> Self {
        let self_pval = self.movie.data().as_ptr() as usize;
        let self_len = self.movie.data().len();
        let slice_pval = slice.as_ptr() as usize;

        if self_pval <= slice_pval && slice_pval < (self_pval + self_len) {
            Self {
                movie: self.movie.clone(),
                start: slice_pval - self_pval,
                end: (slice_pval - self_pval) + slice.len(),
            }
        } else {
            self.copy_empty()
        }
    }

    /// Construct a new SwfSlice from a Reader and a size.
    ///
    /// This is intended to allow constructing references to the contents of a
    /// given SWF tag. You just need the current reader and the size of the tag
    /// you want to reference.
    ///
    /// The returned slice may or may not be a subslice of the current slice.
    /// If the resulting slice would be outside the bounds of the underlying
    /// movie, or the given reader refers to a different underlying movie, this
    /// function returns an empty slice.
    pub fn resize_to_reader(&self, reader: &mut SwfStream<'_>, size: usize) -> Self {
        if self.movie.data().as_ptr() as usize <= reader.get_ref().as_ptr() as usize
            && (reader.get_ref().as_ptr() as usize)
                < self.movie.data().as_ptr() as usize + self.movie.data().len()
        {
            let outer_offset =
                reader.get_ref().as_ptr() as usize - self.movie.data().as_ptr() as usize;
            let new_start = outer_offset;
            let new_end = outer_offset + size;

            let len = self.movie.data().len();

            if new_start < len && new_end < len {
                Self {
                    movie: self.movie.clone(),
                    start: new_start,
                    end: new_end,
                }
            } else {
                self.copy_empty()
            }
        } else {
            self.copy_empty()
        }
    }

    /// Construct a new SwfSlice from a start and an end.
    ///
    /// The start and end values will be relative to the current slice.
    /// Furthermore, this function will yield an empty slice if the calculated slice
    /// would be invalid (e.g. negative length) or would extend past the end of
    /// the current slice.
    pub fn to_start_and_end(&self, start: usize, end: usize) -> Self {
        let new_start = self.start + start;
        let new_end = self.start + end;

        if new_start <= new_end {
            if let Some(result) = self.movie.data().get(new_start..new_end) {
                self.to_subslice(result)
            } else {
                self.copy_empty()
            }
        } else {
            self.copy_empty()
        }
    }

    /// Convert the SwfSlice into a standard data slice.
    pub fn data(&self) -> &[u8] {
        &self.movie.data()[self.start..self.end]
    }

    /// Get the version of the SWF this data comes from.
    pub fn version(&self) -> u8 {
        self.movie.header().version()
    }

    /// Checks if this slice is empty
    pub fn is_empty(&self) -> bool {
        self.end == self.start
    }

    /// Construct a reader for this slice.
    ///
    /// The `from` parameter is the offset to start reading the slice from.
    pub fn read_from(&self, from: u64) -> swf::read::Reader<'_> {
        swf::read::Reader::new(&self.data()[from as usize..], self.movie.version())
    }

    /// Get the length of the SwfSlice.
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}
