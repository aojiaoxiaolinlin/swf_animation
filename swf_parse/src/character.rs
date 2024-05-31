use std::cell::RefCell;

use ruffle_render::bitmap::{BitmapHandle, BitmapSize};
use swf::DefineBitsLossless;

use crate::{
    binary_data::BinaryData,
    display_object::{graphic::Graphic, morph_shape::MorphShape, movie_clip::MovieClip},
};
/// 这个类保存了一个来自 SWF 标签的位图，以及解码后的宽度和高度。我们避免在实际需要之前解压图像
/// ——一些像“House”这样的病态 SWF 文件有数千个高度压缩（大部分为空）的位图，如果在预加载期间解压所有这些位图，可能需要超过 10GB 的内存。
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
#[derive(Clone)]
pub enum Character {
    MovieClip(MovieClip),
    Graphic(Graphic),
    MorphShape(MorphShape),
    Bitmap {
        compressed: CompressedBitmap,
        handle: RefCell<Option<BitmapHandle>>,
    },
    BinaryData(BinaryData),
}
