pub mod codecs;
mod heif;
pub mod icc;
pub mod jpeg;
mod jxl;
mod png;
mod tiff;
mod webp;

use crate::frame::{BitDepth, OutputColorSpace, OutputFormat};
use crate::metadata::ExportMetadata;
use crate::metadata::exif::{self, ExifBlock};

pub use self::heif::{encode_avif_rgb, encode_heic_rgb};
pub use self::jpeg::{encode_jpeg_rgb, encode_jpeg_rgba};
pub use self::jxl::{encode_jxl8, encode_jxl16};
pub use self::png::{encode_png8, encode_png16};
pub use self::tiff::{encode_tiff8, encode_tiff16};
pub use self::webp::encode_webp_rgb;

pub struct ImageRgb8<'a> {
    pub rgb: &'a [u8],
    pub width: u32,
    pub height: u32,
}

pub struct ImageRgba8<'a> {
    pub rgba: &'a [u8],
    pub width: u32,
    pub height: u32,
}

pub fn embedded(
    meta: Option<&ExportMetadata>,
    format: &OutputFormat,
    color_space: OutputColorSpace,
    size: (u32, u32),
) -> (Option<ExifBlock>, Vec<String>) {
    let Some(meta) = meta else {
        return (None, Vec::new());
    };
    let Some(source) = meta.exif.as_ref() else {
        return (
            None,
            vec!["Metadata not copied: no readable EXIF in the original".into()],
        );
    };
    let budget = matches!(format, OutputFormat::Jpeg { .. }).then_some(jpeg::EXIF_BUDGET);
    match exif::block(source, meta.location, budget, color_space, size) {
        Ok(block) => (Some(block), Vec::new()),
        Err(e) => (None, vec![format!("Metadata not copied: {e}")]),
    }
}

pub fn encode_from_rgb8(
    rgb: &[u8],
    width: u32,
    height: u32,
    format: &OutputFormat,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    let img = ImageRgb8 { rgb, width, height };
    match *format {
        OutputFormat::Jpeg {
            quality,
            subsampling,
        } => encode_jpeg_rgb(img, quality as i32, subsampling, cs, exif),
        OutputFormat::Png {
            bit_depth: BitDepth::Eight,
            compression,
        } => encode_png8(img, compression, cs, exif),
        OutputFormat::Png {
            bit_depth: BitDepth::Sixteen,
            compression,
        } => {
            let rgb16: Vec<u16> = rgb.iter().map(|&v| (v as u16) * 257).collect();
            encode_png16(&rgb16, width, height, compression, cs, exif)
        }
        OutputFormat::Webp { quality, lossless } => {
            encode_webp_rgb(img, quality, lossless, cs, exif)
        }
        OutputFormat::Avif { quality } => encode_avif_rgb(img, quality, cs, exif),
        OutputFormat::Heic { quality } => encode_heic_rgb(img, quality, cs, exif),
        OutputFormat::Tiff {
            bit_depth: BitDepth::Eight,
            compression,
        } => encode_tiff8(img, compression, cs, exif),
        OutputFormat::Tiff {
            bit_depth: BitDepth::Sixteen,
            compression,
        } => {
            let rgb16: Vec<u16> = rgb.iter().map(|&v| (v as u16) * 257).collect();
            encode_tiff16(&rgb16, width, height, compression, cs, exif)
        }
        OutputFormat::Jxl {
            bit_depth: BitDepth::Eight,
        } => encode_jxl8(img, cs, exif),
        OutputFormat::Jxl {
            bit_depth: BitDepth::Sixteen,
        } => {
            let rgb16: Vec<u16> = rgb.iter().map(|&v| (v as u16) * 257).collect();
            encode_jxl16(&rgb16, width, height, cs, exif)
        }
        OutputFormat::Rgb8 => Ok(rgb.to_vec()),
    }
}

pub fn encode_from_rgba8(
    rgba: &[u8],
    width: u32,
    height: u32,
    format: &OutputFormat,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    if let OutputFormat::Jpeg {
        quality,
        subsampling,
    } = *format
    {
        return encode_jpeg_rgba(
            ImageRgba8 {
                rgba,
                width,
                height,
            },
            quality as i32,
            subsampling,
            cs,
            exif,
        );
    }
    let mut rgb: Vec<u8> = Vec::with_capacity((width as usize) * (height as usize) * 3);
    for chunk in rgba.chunks_exact(4) {
        rgb.extend_from_slice(&chunk[..3]);
    }
    encode_from_rgb8(&rgb, width, height, format, cs, exif)
}

pub fn encode_from_rgb16(
    rgb16: &[u16],
    width: u32,
    height: u32,
    format: &OutputFormat,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    match *format {
        OutputFormat::Png {
            bit_depth: BitDepth::Sixteen,
            compression,
        } => encode_png16(rgb16, width, height, compression, cs, exif),
        OutputFormat::Tiff {
            bit_depth: BitDepth::Sixteen,
            compression,
        } => encode_tiff16(rgb16, width, height, compression, cs, exif),
        OutputFormat::Jxl {
            bit_depth: BitDepth::Sixteen,
        } => encode_jxl16(rgb16, width, height, cs, exif),
        _ => {
            let rgb8: Vec<u8> = rgb16.iter().map(|&v| (v >> 8) as u8).collect();
            encode_from_rgb8(&rgb8, width, height, format, cs, exif)
        }
    }
}

#[cfg(test)]
mod tests;
