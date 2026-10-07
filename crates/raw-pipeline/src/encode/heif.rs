use libheif_rs::{
    Channel, ColorProfileRaw, ColorSpace, CompressionFormat, EncoderQuality, HeifContext,
    Image as HeifImage, LibHeif, RgbChroma, color_profile_types,
};

use super::ImageRgb8;
use crate::PipelineError;
use crate::frame::OutputColorSpace;
use crate::metadata::exif::ExifBlock;

fn heif_encoder_hint(format: CompressionFormat) -> &'static str {
    match format {
        CompressionFormat::Hevc => "HEVC encoder plugin missing (install libheif-plugin-x265)",
        CompressionFormat::Av1 => {
            "AV1 encoder plugin missing (install libheif-plugin-aomenc or libheif-plugin-rav1e)"
        }
        _ => "libheif encoder plugin missing for this format",
    }
}

fn encode_heif_rgb(
    img: ImageRgb8<'_>,
    quality: u8,
    format: CompressionFormat,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    let lib_heif = LibHeif::new();
    let mut heif_image = HeifImage::new(img.width, img.height, ColorSpace::Rgb(RgbChroma::Rgb))
        .map_err(|e| PipelineError::Encode(format!("heif image: {e}")))?;
    heif_image
        .create_plane(Channel::Interleaved, img.width, img.height, 8)
        .map_err(|e| PipelineError::Encode(format!("heif plane: {e}")))?;
    let icc_profile = ColorProfileRaw::new(color_profile_types::PROF, cs.icc_profile().to_vec());
    heif_image
        .set_color_profile_raw(&icc_profile)
        .map_err(|e| PipelineError::Encode(format!("heif icc: {e}")))?;
    {
        let planes = heif_image.planes_mut();
        let plane = planes
            .interleaved
            .ok_or_else(|| PipelineError::Encode("heif: no interleaved plane".into()))?;
        let stride = plane.stride;
        let row_bytes = img.width as usize * 3;
        for y in 0..img.height as usize {
            let dst_off = y * stride;
            let src_off = y * row_bytes;
            plane.data[dst_off..dst_off + row_bytes]
                .copy_from_slice(&img.rgb[src_off..src_off + row_bytes]);
        }
    }
    let mut encoder = lib_heif.encoder_for_format(format).map_err(|e| {
        PipelineError::Encode(format!("heif encoder: {e}; {}", heif_encoder_hint(format)))
    })?;
    encoder
        .set_quality(EncoderQuality::Lossy(quality))
        .map_err(|e| PipelineError::Encode(format!("heif quality: {e}")))?;
    let mut ctx =
        HeifContext::new().map_err(|e| PipelineError::Encode(format!("heif ctx: {e}")))?;
    let handle = ctx
        .encode_image(&heif_image, &mut encoder, None)
        .map_err(|e| PipelineError::Encode(format!("heif encode: {e}")))?;
    if let Some(exif) = exif {
        ctx.add_exif_metadata(&handle, &exif.tiff)
            .map_err(|e| PipelineError::Encode(format!("heif exif: {e}")))?;
    }
    ctx.write_to_bytes()
        .map_err(|e| PipelineError::Encode(format!("heif write: {e}")))
}

pub fn encode_avif_rgb(
    img: ImageRgb8<'_>,
    quality: u8,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    encode_heif_rgb(img, quality, CompressionFormat::Av1, cs, exif)
}

pub fn encode_heic_rgb(
    img: ImageRgb8<'_>,
    quality: u8,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    encode_heif_rgb(img, quality, CompressionFormat::Hevc, cs, exif)
}
