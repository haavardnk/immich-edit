use super::{ImageRgb8, ImageRgba8};
use crate::PipelineError;
use crate::frame::{JpegSubsampling, OutputColorSpace};
use crate::metadata::exif::ExifBlock;

const SOI: [u8; 2] = [0xFF, 0xD8];
const APP1: u8 = 0xE1;
const APP2: u8 = 0xE2;
const SEGMENT_MAX: usize = u16::MAX as usize - 2;
const EXIF_SIGNATURE: &[u8] = b"Exif\0\0";
const ICC_SIGNATURE: &[u8] = b"ICC_PROFILE\0";
const ICC_CHUNK: usize = SEGMENT_MAX - ICC_SIGNATURE.len() - 2;

pub const EXIF_BUDGET: usize = SEGMENT_MAX - EXIF_SIGNATURE.len();

fn turbo_subsamp(subsampling: JpegSubsampling) -> turbojpeg::Subsamp {
    match subsampling {
        JpegSubsampling::Chroma420 => turbojpeg::Subsamp::Sub2x2,
        JpegSubsampling::Chroma444 => turbojpeg::Subsamp::None,
    }
}

pub fn encode_jpeg_rgb(
    img: ImageRgb8<'_>,
    quality: i32,
    subsampling: JpegSubsampling,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    let image = turbojpeg::Image {
        pixels: img.rgb,
        width: img.width as usize,
        pitch: img.width as usize * 3,
        height: img.height as usize,
        format: turbojpeg::PixelFormat::RGB,
    };
    compress(image, quality, subsampling, cs, exif)
}

pub fn encode_jpeg_rgba(
    img: ImageRgba8<'_>,
    quality: i32,
    subsampling: JpegSubsampling,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    let image = turbojpeg::Image {
        pixels: img.rgba,
        width: img.width as usize,
        pitch: img.width as usize * 4,
        height: img.height as usize,
        format: turbojpeg::PixelFormat::RGBA,
    };
    compress(image, quality, subsampling, cs, exif)
}

fn compress(
    image: turbojpeg::Image<&[u8]>,
    quality: i32,
    subsampling: JpegSubsampling,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    let jpeg = turbojpeg::compress(image, quality, turbo_subsamp(subsampling))
        .map_err(|e| PipelineError::Encode(format!("{e}")))?;
    let Some(body) = jpeg.strip_prefix(&SOI) else {
        return Err(PipelineError::Encode(
            "jpeg: encoder output lacks SOI".into(),
        ));
    };
    let mut out = Vec::with_capacity(jpeg.len() + 4096);
    out.extend_from_slice(&SOI);
    if let Some(exif) = exif {
        segment(&mut out, APP1, &[EXIF_SIGNATURE, &exif.tiff])?;
    }
    let icc = cs.icc_profile();
    let total = icc.len().div_ceil(ICC_CHUNK);
    for (index, chunk) in icc.chunks(ICC_CHUNK).enumerate() {
        let sequence = [index as u8 + 1, total as u8];
        segment(&mut out, APP2, &[ICC_SIGNATURE, &sequence, chunk])?;
    }
    out.extend_from_slice(body);
    Ok(out)
}

fn segment(out: &mut Vec<u8>, marker: u8, parts: &[&[u8]]) -> crate::PipelineResult<()> {
    let len: usize = parts.iter().map(|p| p.len()).sum();
    if len > SEGMENT_MAX {
        return Err(PipelineError::Encode(format!(
            "jpeg: {len}-byte APP{} payload exceeds the segment limit",
            marker - 0xE0
        )));
    }
    out.extend_from_slice(&[0xFF, marker]);
    out.extend_from_slice(&((len + 2) as u16).to_be_bytes());
    parts.iter().for_each(|p| out.extend_from_slice(p));
    Ok(())
}
