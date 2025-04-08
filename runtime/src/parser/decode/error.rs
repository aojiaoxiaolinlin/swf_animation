use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Bitmap texture is larger than the rendering device supports")]
    TooLarge,

    #[error("Unknown bitmap format")]
    UnknownType,

    #[error("Invalid JPEG")]
    InvalidJpeg(#[from] jpeg_decoder::Error),

    #[error("Invalid PNG")]
    InvalidPng(#[from] png::DecodingError),

    #[error("Invalid GIF")]
    InvalidGif(#[from] gif::DecodingError),
}
