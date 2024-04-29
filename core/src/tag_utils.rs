use std::sync::Arc;

use swf::{HeaderExt, TagCode};
use swf::error::Error;
use url::Url;

#[derive(Debug, Clone)]
pub struct SwfMovie {
    /// SWF 文件的头部
    header: HeaderExt,
    /// 解压后的数据
    data: Vec<u8>,

    /// SWF 文件的原始url
    url: String,

    /// 编码
    encoding: &'static swf::Encoding,

    /// 整个数据流的压缩长度
    compressed_len: usize,

    /// 此 SwfMovie 是否真正代表已加载的影片，还是填充其他内容，
    /// 如已加载的图像、填充影片或错误状态。
    is_movie: bool,
}
impl SwfMovie {
    pub fn new_empty(swf_version: u8) -> Self {
        Self {
            header: swf::HeaderExt::default_with_swf_version(swf_version),
            data: Vec::new(),
            url: "file:///".into(),
            encoding: swf::UTF_8,
            compressed_len: 0,
            is_movie: false,
        }
    }

    pub fn from_path(url:&Url)->Result<SwfMovie, Error> {
        let data = std::fs::read(url.path().strip_prefix("/").unwrap()).unwrap();
        Self::from_data(&data,&url)
    }

    pub fn from_data(swf_data:&[u8],
    url:&Url)->Result<Self,Error>{
        let compressed_len = swf_data.len();
        let swf_buf = swf::read::decompress_swf(swf_data)?;
        let encoding = swf::SwfStr::encoding_for_version(swf_buf.header.version());
        let swf_movie = SwfMovie {
            header: swf_buf.header,
            data: swf_buf.data,
            url: url.to_string(),
            encoding,
            compressed_len,
            is_movie: true,
        };
        Ok(swf_movie)
    }

    pub fn header(&self) -> &HeaderExt {
        &self.header
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    pub fn version(&self) -> u8 {
        self.header.version()
    }
}


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
pub enum ControlFlow {
    /// Stop decoding after this tag.
    Exit,

    /// Continue decoding the next tag.
    Continue,
}

pub type DecodeResult = Result<ControlFlow, Error>;
pub type SwfStream<'a> = swf::read::Reader<'a>;

/// Decode tags from a SWF stream reader.
///
/// The given `tag_callback` will be called for each decoded tag. It will be
/// provided with the stream to read from, the tag code read, and the tag's
/// size. The callback is responsible for (optionally) parsing the contents of
/// the tag; otherwise, it will be skipped.
///
/// Decoding will terminate when the following conditions occur:
///
///  * The `tag_callback` calls for the decoding to finish.
///  * The decoder encounters a tag longer than the underlying SWF slice
///    (indicated by returning false)
///  * The SWF stream is otherwise corrupt or unreadable (indicated as an error
///    result)
///
/// Decoding will also log tags longer than the SWF slice, error messages
/// yielded from the tag callback, and unknown tags. It will *only* return an
/// error message if the SWF tag itself could not be parsed. Other forms of
/// irregular decoding will be signalled by returning false.
pub fn decode_tags<'a, F>(reader: &mut SwfStream<'a>, mut tag_callback: F) -> Result<bool, Error>
where
    F: for<'b> FnMut(&'b mut SwfStream<'a>, TagCode, usize) -> Result<ControlFlow, Error>,
{
    loop {
        let (tag_code, tag_len) = reader.read_tag_code_and_length()?;
        if tag_len > reader.get_ref().len() {
            // tracing::error!("Unexpected EOF when reading tag");
            *reader.get_mut() = &reader.get_ref()[reader.get_ref().len()..];
            return Ok(false);
        }

        let tag_slice = &reader.get_ref()[..tag_len];
        let end_slice = &reader.get_ref()[tag_len..];
        if let Some(tag) = TagCode::from_u16(tag_code) {
            *reader.get_mut() = tag_slice;
            let result = tag_callback(reader, tag, tag_len);

            match result {
                Err(e) => {
                    // tracing::error!("Error running definition tag: {:?}, got {}", tag, e)
                }
                Ok(ControlFlow::Exit) => {
                    *reader.get_mut() = end_slice;
                    break;
                }
                Ok(ControlFlow::Continue) => {}
            }
        } else {
            // tracing::warn!("Unknown tag code: {:?}", tag_code);
        }

        *reader.get_mut() = end_slice;
    }

    Ok(true)
}
