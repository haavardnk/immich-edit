use crate::frame::{BitDepth, JpegSubsampling, OutputFormat, PngCompression, TiffCompression};

const MIB: u64 = 1024 * 1024;
pub(crate) const RGB8: u64 = 3;
pub(crate) const RGBA8: u64 = 4;
pub(crate) const RGB16: u64 = 6;

#[derive(Clone, Copy)]
pub(crate) enum EncodeInput {
    Rgb8,
    Rgba8,
    Rgb16,
}

pub(crate) fn pixel_bytes((w, h): (u32, u32), per_px: u64) -> u64 {
    u64::from(w) * u64::from(h) * per_px
}

pub(crate) fn encode_bytes(input: EncodeInput, format: &OutputFormat, dims: (u32, u32)) -> u64 {
    let sixteen = format.bit_depth() == BitDepth::Sixteen;
    let converted = match input {
        EncodeInput::Rgba8 if matches!(format, OutputFormat::Jpeg { .. }) => 0,
        EncodeInput::Rgba8 if sixteen => RGB8 + RGB16,
        EncodeInput::Rgba8 => RGB8,
        EncodeInput::Rgb8 if sixteen => RGB16,
        EncodeInput::Rgb16 if !sixteen => RGB8,
        EncodeInput::Rgb8 | EncodeInput::Rgb16 => 0,
    };
    pixel_bytes(dims, converted) + working_bytes(format, dims)
}

pub fn working_bytes(format: &OutputFormat, dims: (u32, u32)) -> u64 {
    let (fixed_mib, per_px) = match *format {
        OutputFormat::Rgb8 => (0, 4),
        OutputFormat::Jpeg {
            subsampling: JpegSubsampling::Chroma420,
            ..
        } => (0, 4),
        OutputFormat::Jpeg { .. } => (0, 7),
        OutputFormat::Png {
            bit_depth,
            compression,
        } => match (bit_depth, compression == PngCompression::Fast) {
            (BitDepth::Eight, true) => (2, 5),
            (BitDepth::Eight, false) => (2, 4),
            (BitDepth::Sixteen, true) => (2, 10),
            (BitDepth::Sixteen, false) => (2, 7),
        },
        OutputFormat::Tiff {
            bit_depth,
            compression,
        } => match (bit_depth, compression == TiffCompression::Lzw) {
            (BitDepth::Eight, true) => (16, 6),
            (BitDepth::Eight, false) => (2, 4),
            (BitDepth::Sixteen, true) => (16, 11),
            (BitDepth::Sixteen, false) => (2, 7),
        },
        OutputFormat::Jxl {
            bit_depth: BitDepth::Eight,
        } => (340, 9),
        OutputFormat::Jxl { .. } => (360, 10),
        OutputFormat::Webp { lossless: true, .. } => (0, 43),
        OutputFormat::Webp { .. } => (8, 29),
        OutputFormat::Avif { .. } => (16, 30),
        OutputFormat::Heic { .. } => (60, 33),
    };
    fixed_mib * MIB + pixel_bytes(dims, per_px)
}
