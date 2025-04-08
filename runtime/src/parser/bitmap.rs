use swf::DefineBitsLossless;

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
            CompressedBitmap::Lossless(DefineBitsLossless { width, height, .. }) => BitmapSize {
                width: *width,
                height: *height,
            },
        }
    }
}

pub struct BitmapSize {
    pub width: u16,
    pub height: u16,
}
