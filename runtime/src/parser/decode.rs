mod error;

use error::Error;
use std::borrow::Cow;

/// The format of image data in a DefineBitsJpeg2/3 tag.
/// Generally this will be JPEG, but according to SWF19, these tags can also contain PNG and GIF data.
/// SWF19 pp.138-139
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum JpegTagFormat {
    Jpeg,
    Png,
    Gif,
    Unknown,
}

/// Determines the format of the image data in `data` from a DefineBitsJPEG2/3 tag.
pub fn determine_jpeg_tag_format(data: &[u8]) -> JpegTagFormat {
    match data {
        [0xff, 0xd8, ..] => JpegTagFormat::Jpeg,
        [0xff, 0xd9, 0xff, 0xd8, ..] => JpegTagFormat::Jpeg, // erroneous header in SWF
        [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, ..] => JpegTagFormat::Png,
        [0x47, 0x49, 0x46, 0x38, 0x39, 0x61, ..] => JpegTagFormat::Gif,
        _ => JpegTagFormat::Unknown,
    }
}

pub fn decode_define_bits_jpeg_dimensions(data: &[u8]) -> Result<(u16, u16), Error> {
    let format = determine_jpeg_tag_format(data);
    match format {
        JpegTagFormat::Jpeg => decode_jpeg_dimensions(data),
        JpegTagFormat::Png => decode_png_dimensions(data),
        JpegTagFormat::Gif => decode_gif_dimensions(data),
        JpegTagFormat::Unknown => Err(Error::UnknownType),
    }
}

/// Removes potential invalid JPEG data from SWF DefineBitsJPEG tags.
/// These bytes need to be removed for the JPEG to decode properly.
pub fn remove_invalid_jpeg_data(data: &[u8]) -> Cow<[u8]> {
    // SWF19 errata p.138:
    // "Before version 8 of the SWF file format, SWF files could contain an erroneous header of 0xFF, 0xD9, 0xFF, 0xD8
    // before the JPEG SOI marker."
    // 0xFFD9FFD8 is a JPEG EOI+SOI marker pair. Contrary to the spec, this invalid marker sequence can actually appear
    // at any time before the 0xFFC0 SOF marker, not only at the beginning of the data. I believe this is a relic from
    // the SWF JPEGTables tag, which stores encoding tables separately from the DefineBits image data, encased in its
    // own SOI+EOI pair. When these data are glued together, an interior EOI+SOI sequence is produced. The Flash JPEG
    // decoder expects this pair and ignores it, despite standard JPEG decoders stopping at the EOI.
    // When DefineBitsJPEG2 etc. were introduced, the Flash encoders/decoders weren't properly adjusted, resulting in
    // this sequence persisting. Also, despite what the spec says, this doesn't appear to be version checked (e.g., a
    // v9 SWF can contain one of these malformed JPEGs and display correctly).
    // See https://github.com/ruffle-rs/ruffle/issues/8775 for various examples.

    // JPEG markers
    const SOF0: u8 = 0xC0; // Start of frame
    const RST0: u8 = 0xD0; // Restart (we shouldn't see this before SOS, but just in case)
    const RST7: u8 = 0xD7;
    const SOI: u8 = 0xD8; // Start of image
    const EOI: u8 = 0xD9; // End of image

    let mut data: Cow<[u8]> = if let Some(stripped) = data.strip_prefix(&[0xFF, EOI, 0xFF, SOI]) {
        // Common case: usually the sequence is at the beginning as the spec says, so adjust the slice to avoid a copy.
        stripped.into()
    } else {
        // Parse the JPEG markers searching for the 0xFFD9FFD8 marker sequence to splice out.
        // We only have to search up to the SOF0 marker.
        // This might be another case where eventually we want to write our own full JPEG decoder to match Flash's decoder.
        let mut jpeg_data = data;
        let mut pos = 0;
        loop {
            if jpeg_data.len() < 4 {
                // No invalid sequence found before SOF marker, return data as-is.
                break data.into();
            }
            let payload_len: usize = match &jpeg_data[..4] {
                [0xFF, EOI, 0xFF, SOI] => {
                    // Invalid EOI+SOI sequence found, splice it out.
                    let mut out_data = Vec::with_capacity(data.len() - 4);
                    out_data.extend_from_slice(&data[..pos]);
                    out_data.extend_from_slice(&data[pos + 4..]);
                    break out_data.into();
                }
                // EOI, SOI, RST markers do not include a size.
                [0xFF, EOI | SOI | RST0..=RST7, _, _] => 0,
                [0xFF, SOF0, _, _] => {
                    // No invalid sequence found before SOF marker, return data as-is.
                    break data.into();
                }
                // Other tags include a length.
                [0xFF, _, a, b] => u16::from_be_bytes([*a, *b]).into(),
                _ => {
                    // All JPEG markers should start with 0xFF.
                    // So this is either not a JPEG, or we screwed up parsing the markers. Bail out.
                    break data.into();
                }
            };
            // Advance to next JPEG marker.
            jpeg_data = jpeg_data.get(payload_len + 2..).unwrap_or_default();
            pos += payload_len + 2;
        }
    };

    // Some JPEGs are missing the final EOI marker (JPEG optimizers truncate it?)
    // Flash and most image decoders will still display these images, but jpeg-decoder errors.
    // Glue on an EOI marker if its not already there and hope for the best.
    if data.ends_with(&[0xFF, EOI]) {
        data
    } else {
        tracing::warn!("JPEG is missing EOI marker and may not decode properly");
        data.to_mut().extend_from_slice(&[0xFF, EOI]);
        data
    }
}

/// Some SWFs report unreasonable bitmap dimensions (#1191).
/// Fail before decoding such bitmaps to avoid panics.
fn validate_size(width: u16, height: u16) -> Result<(), Error> {
    const INVALID_SIZE: usize = 0x8000000; // 128MB

    let size = (width as usize).saturating_mul(height as usize);
    if size >= INVALID_SIZE {
        return Err(Error::TooLarge);
    }
    Ok(())
}

fn decode_jpeg_dimensions(jpeg_data: &[u8]) -> Result<(u16, u16), Error> {
    let jpeg_data = remove_invalid_jpeg_data(jpeg_data);

    let mut decoder = jpeg_decoder::Decoder::new(&jpeg_data[..]);
    decoder.read_info()?;
    let metadata = decoder
        .info()
        .expect("info() should always return Some if read_info returned Ok");
    validate_size(metadata.width, metadata.height)?;
    Ok((metadata.width, metadata.height))
}

fn decode_png_dimensions(data: &[u8]) -> Result<(u16, u16), Error> {
    use png::Transformations;

    let mut decoder = png::Decoder::new(data);
    // Normalize output to 8-bit grayscale or RGB.
    // Ideally we'd want to normalize to 8-bit RGB only, but seems like the `png` crate provides no such a feature.
    decoder.set_transformations(Transformations::normalize_to_color8());
    let reader = decoder.read_info()?;
    Ok((
        reader.info().width.try_into().expect("Invalid PNG width"),
        reader.info().height.try_into().expect("Invalid PNG height"),
    ))
}

fn decode_gif_dimensions(data: &[u8]) -> Result<(u16, u16), Error> {
    let mut decode_options = gif::DecodeOptions::new();
    decode_options.set_color_output(gif::ColorOutput::RGBA);
    let reader = decode_options.read_info(data)?;
    Ok((reader.width(), reader.height()))
}
