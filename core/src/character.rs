use std::cell::RefCell;

// use crate::backend::audio::SoundHandle;
// use crate::binary_data::BinaryData;
// use crate::display_object::{
//     Graphic, MorphShape, MovieClip, Text, Video,
// };
// use crate::font::Font;
use ruffle_render::bitmap::{BitmapHandle, BitmapSize};
use swf::DefineBitsLossless;

#[derive(Clone, Debug)]
pub enum Character {
    // EditText(EditText),
    // Graphic(Graphic),
    // MovieClip(MovieClip),
    // Bitmap {
    //     compressed: CompressedBitmap,
    //     /// A lazily constructed GPU handle, used when performing fills with this bitmap
    //     handle: RefCell<Option<BitmapHandle>>,

    // },
    // // Font(Font),
    // MorphShape(MorphShape),
    // Text(Text),
    // Video(Video),
    // BinaryData(BinaryData),
}

/// Holds a bitmap from an SWF tag, plus the decoded width/height.
/// We avoid decompressing the image until it's actually needed - some pathological SWFS
/// like 'House' have thousands of highly-compressed (mostly empty) bitmaps, which can
/// take over 10GB of ram if we decompress them all during preloading.
#[derive(Clone, Debug)]
pub enum CompressedBitmap {
    Jpeg {
        data: Vec<u8>,
        alpha: Option<Vec<u8>>,
        width: u16,
        height: u16,
    },
    Lossless(DefineBitsLossless<'static>),
}

impl CompressedBitmap {
    pub fn size(&self) -> BitmapSize {
        match self {
            CompressedBitmap::Jpeg { width, height, .. } => BitmapSize {
                width: *width,
                height: *height,
            },
            CompressedBitmap::Lossless(define_bits_lossless) => BitmapSize {
                width: define_bits_lossless.width,
                height: define_bits_lossless.height,
            },
        }
    }
    pub fn decode(&self) -> Result<ruffle_render::bitmap::Bitmap, ruffle_render::error::Error> {
        match self {
            CompressedBitmap::Jpeg {
                data,
                alpha,
                width: _,
                height: _,
            } => ruffle_render::utils::decode_define_bits_jpeg(data, alpha.as_deref()),
            CompressedBitmap::Lossless(define_bits_lossless) => {
                ruffle_render::utils::decode_define_bits_lossless(define_bits_lossless)
            }
        }
    }
}
