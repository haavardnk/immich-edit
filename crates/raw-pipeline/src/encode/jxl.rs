use jpegxl_rs::encode::{ColorEncoding, JxlEncoder, Metadata};

use super::ImageRgb8;
use crate::PipelineError;
use crate::frame::OutputColorSpace;
use crate::metadata::exif::ExifBlock;

const EXIF_TIFF_OFFSET: [u8; 4] = [0; 4];

pub fn encode_jxl8(
    img: ImageRgb8<'_>,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    let mut encoder = encoder(cs, exif)?;
    let result: jpegxl_rs::encode::EncoderResult<u8> = encoder
        .encode::<u8, u8>(img.rgb, img.width, img.height)
        .map_err(|e| PipelineError::Encode(format!("jxl: {e}")))?;
    Ok(result.data)
}

pub fn encode_jxl16(
    rgb16: &[u16],
    width: u32,
    height: u32,
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<Vec<u8>> {
    let mut encoder = encoder(cs, exif)?;
    let result: jpegxl_rs::encode::EncoderResult<u16> = encoder
        .encode::<u16, u16>(rgb16, width, height)
        .map_err(|e| PipelineError::Encode(format!("jxl: {e}")))?;
    Ok(result.data)
}

fn encoder(
    cs: OutputColorSpace,
    exif: Option<&ExifBlock>,
) -> crate::PipelineResult<JxlEncoder<'static, 'static>> {
    let mut encoder = jpegxl_rs::encoder_builder()
        .color_encoding(jxl_color_encoding(cs))
        .build()
        .map_err(|e| PipelineError::Encode(format!("jxl: {e}")))?;
    if let Some(exif) = exif {
        let data = [EXIF_TIFF_OFFSET.as_slice(), &exif.tiff].concat();
        encoder
            .add_metadata(&Metadata::Exif(&data), false)
            .map_err(|e| PipelineError::Encode(format!("jxl exif: {e}")))?;
    }
    Ok(encoder)
}

fn jxl_color_encoding(cs: OutputColorSpace) -> ColorEncoding {
    use jpegxl_sys::color::color_encoding::{
        JxlColorEncoding, JxlColorSpace, JxlPrimaries, JxlRenderingIntent, JxlTransferFunction,
        JxlWhitePoint,
    };
    match cs {
        OutputColorSpace::SRgb => ColorEncoding::Srgb,
        OutputColorSpace::DisplayP3 => ColorEncoding::Custom(JxlColorEncoding {
            color_space: JxlColorSpace::Rgb,
            white_point: JxlWhitePoint::D65,
            white_point_xy: [0.3127, 0.3290],
            primaries: JxlPrimaries::P3,
            primaries_red_xy: [0.680, 0.320],
            primaries_green_xy: [0.265, 0.690],
            primaries_blue_xy: [0.150, 0.060],
            transfer_function: JxlTransferFunction::SRGB,
            gamma: 0.0,
            rendering_intent: JxlRenderingIntent::Perceptual,
        }),
    }
}
