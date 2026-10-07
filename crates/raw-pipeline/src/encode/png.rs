use std::borrow::Cow;

use super::ImageRgb8;
use crate::PipelineError;
use crate::frame::{OutputColorSpace, PngCompression};
use crate::metadata::exif::ExifBlock;

pub fn encode_png8(
    img: ImageRgb8<'_>,
    compression: PngCompression,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    write(
        img.width,
        img.height,
        png::BitDepth::Eight,
        img.rgb,
        compression,
        cs,
        exif,
    )
}

pub fn encode_png16(
    rgb16: &[u16],
    width: u32,
    height: u32,
    compression: PngCompression,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    let be: Vec<u8> = rgb16.iter().flat_map(|v| v.to_be_bytes()).collect();
    write(
        width,
        height,
        png::BitDepth::Sixteen,
        &be,
        compression,
        cs,
        exif,
    )
}

fn write(
    width: u32,
    height: u32,
    depth: png::BitDepth,
    data: &[u8],
    compression: PngCompression,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    let mut info = png::Info::with_size(width, height);
    info.color_type = png::ColorType::Rgb;
    info.bit_depth = depth;
    info.exif_metadata = exif.map(|e| Cow::Borrowed(e.tiff.as_slice()));
    if cs != OutputColorSpace::SRgb {
        info.icc_profile = Some(Cow::Borrowed(cs.icc_profile()));
    }
    let mut buf: Vec<u8> = Vec::new();
    let mut enc = png::Encoder::with_info(&mut buf, info)
        .map_err(|e| PipelineError::Encode(format!("png: {e}")))?;
    enc.set_compression(map_png_compression(compression));
    if cs == OutputColorSpace::SRgb {
        enc.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    }
    let mut writer = enc
        .write_header()
        .map_err(|e| PipelineError::Encode(format!("png: {e}")))?;
    writer
        .write_image_data(data)
        .map_err(|e| PipelineError::Encode(format!("png: {e}")))?;
    writer
        .finish()
        .map_err(|e| PipelineError::Encode(format!("png: {e}")))?;
    Ok(buf)
}

fn map_png_compression(c: PngCompression) -> png::Compression {
    match c {
        PngCompression::Fast => png::Compression::Fast,
        PngCompression::Default => png::Compression::Balanced,
        PngCompression::Best => png::Compression::High,
    }
}
