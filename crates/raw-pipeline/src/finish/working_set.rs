use super::enlarged_dims;
use super::watermark::overlay_dims;
use crate::encode::{EncodeInput, RGB8, RGB16, RGBA8, encode_bytes, pixel_bytes};
use crate::frame::{RenderOptions, Watermark};

const OUTPUT_SHARPEN: u64 = 12;
const OVERLAY: u64 = 16;

pub(crate) fn finish_bytes(sixteen: bool, out: (u32, u32), options: &RenderOptions) -> u64 {
    let texel = if sixteen { RGB16 } else { RGB8 };
    let (enlarge, grown, last) = match enlarged_dims(out.0, out.1, options) {
        Some(enlarged) => {
            let staging = if sixteen {
                pixel_bytes(out, texel) + pixel_bytes(enlarged, texel)
            } else {
                0
            };
            let resized = pixel_bytes((enlarged.0, out.1), texel) + pixel_bytes(enlarged, texel);
            let grown = pixel_bytes(enlarged, texel).saturating_sub(pixel_bytes(out, texel));
            (staging + resized, grown, enlarged)
        }
        None => (0, 0, out),
    };
    let sharpen = if options.output_sharpen.is_some() {
        pixel_bytes(last, OUTPUT_SHARPEN)
    } else {
        0
    };
    let watermark = options
        .watermark
        .as_ref()
        .map_or(0, |mark| watermark_bytes(mark, last));
    let input = if sixteen {
        EncodeInput::Rgb16
    } else {
        EncodeInput::Rgb8
    };
    let encode = encode_bytes(input, &options.output, last);
    enlarge.max(grown + sharpen.max(watermark).max(encode))
}

fn watermark_bytes(watermark: &Watermark, dims: (u32, u32)) -> u64 {
    let mark = overlay_dims(watermark, dims);
    let source = (watermark.image.width, watermark.image.height);
    let resized = pixel_bytes((mark.0, source.1), RGBA8) + pixel_bytes(mark, RGBA8);
    pixel_bytes(source, RGBA8) + resized + pixel_bytes(mark, OVERLAY)
}
