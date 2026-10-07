use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;

use super::*;
use crate::frame::{JpegSubsampling, PngCompression, TiffCompression};

fn red_pixels(w: u32, h: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 3) as usize);
    for _ in 0..(w * h) {
        v.extend_from_slice(&[200, 50, 50]);
    }
    v
}

fn contains_icc(bytes: &[u8]) -> bool {
    bytes.windows(4).any(|w| w == b"acsp")
}

fn contains_profile(bytes: &[u8], profile: &[u8]) -> bool {
    let head = &profile[..profile.len().min(48)];
    bytes.windows(head.len()).any(|w| w == head)
}

fn tagged() -> ExifBlock {
    let mut source = Metadata::new();
    source.set_tag(ExifTag::Make("SONY".to_string()));
    source.set_tag(ExifTag::Model("ILCE-7M4".to_string()));
    source.set_tag(ExifTag::GPSLatitude(vec![59u32.into(); 3]));
    source.set_tag(ExifTag::GPSLatitudeRef("N".to_string()));
    exif::block(&source, true, None, OutputColorSpace::SRgb, (16, 16)).unwrap()
}

fn every_format() -> Vec<OutputFormat> {
    let mut formats = vec![
        OutputFormat::Jpeg {
            quality: 85,
            subsampling: JpegSubsampling::Chroma420,
        },
        OutputFormat::Webp {
            quality: 85,
            lossless: false,
        },
        OutputFormat::Webp {
            quality: 85,
            lossless: true,
        },
        OutputFormat::Avif { quality: 60 },
        OutputFormat::Heic { quality: 60 },
    ];
    for bit_depth in [BitDepth::Eight, BitDepth::Sixteen] {
        formats.push(OutputFormat::Png {
            bit_depth,
            compression: PngCompression::Fast,
        });
        formats.push(OutputFormat::Tiff {
            bit_depth,
            compression: TiffCompression::Lzw,
        });
        formats.push(OutputFormat::Jxl { bit_depth });
    }
    formats
}

#[test]
fn every_format_carries_exif() {
    let rgb = red_pixels(16, 16);
    let exif = tagged();
    let failures: Vec<String> = every_format()
        .iter()
        .filter_map(|format| {
            let out = encode_from_rgb8(&rgb, 16, 16, format, OutputColorSpace::SRgb, Some(&exif))
                .unwrap();
            let tags = crate::metadata::read(&out).unwrap_or_default();
            let make = (&tags)
                .into_iter()
                .any(|t| matches!(t, ExifTag::Make(v) if v == "SONY"));
            let gps = (&tags)
                .into_iter()
                .any(|t| matches!(t, ExifTag::GPSLatitude(_)));
            (!(make && gps)).then(|| format!("{format:?}: make {make} gps {gps}"))
        })
        .collect();
    if !failures.is_empty() {
        panic!("metadata lost:\n{}", failures.join("\n"));
    }
}

#[test]
fn webp_wraps_lossy_and_lossless_into_vp8x() {
    let rgb = red_pixels(20, 12);
    let exif = tagged();
    for lossless in [false, true] {
        let out = encode_webp_rgb(
            ImageRgb8 {
                rgb: &rgb,
                width: 20,
                height: 12,
            },
            85,
            lossless,
            OutputColorSpace::SRgb,
            Some(&exif),
        )
        .unwrap();
        let riff = u32::from_le_bytes([out[4], out[5], out[6], out[7]]) as usize;
        let width = u32::from_le_bytes([out[24], out[25], out[26], 0]) + 1;
        let height = u32::from_le_bytes([out[27], out[28], out[29], 0]) + 1;
        if &out[12..16] != b"VP8X" || out[20] & 0x28 != 0x28 || riff + 8 != out.len() {
            panic!("lossless {lossless}: bad VP8X header or RIFF size");
        }
        if (width, height) != (20, 12) {
            panic!("lossless {lossless}: VP8X says {width}x{height}");
        }
    }
}

#[test]
fn jpeg_embeds_icc() {
    let rgb = red_pixels(32, 32);
    let out = encode_jpeg_rgb(
        ImageRgb8 {
            rgb: &rgb,
            width: 32,
            height: 32,
        },
        85,
        JpegSubsampling::Chroma420,
        OutputColorSpace::SRgb,
        None,
    )
    .unwrap();
    if !(out[0] == 0xFF && out[1] == 0xD8 && out[2] == 0xFF && out[3] == 0xE2) {
        panic!("expected APP2 right after SOI");
    }
    if &out[6..18] != b"ICC_PROFILE\0" {
        panic!("expected ICC_PROFILE marker");
    }
    if !contains_icc(&out) {
        panic!("missing ICC body");
    }
}

#[test]
fn jpeg_embeds_display_p3_profile() {
    let rgb = red_pixels(16, 16);
    let out = encode_jpeg_rgb(
        ImageRgb8 {
            rgb: &rgb,
            width: 16,
            height: 16,
        },
        85,
        JpegSubsampling::Chroma420,
        OutputColorSpace::DisplayP3,
        None,
    )
    .unwrap();
    if !contains_profile(&out, icc::DISPLAY_P3_ICC) {
        panic!("jpeg missing Display P3 profile");
    }
}

#[test]
fn png_embeds_display_p3_iccp() {
    let rgb = red_pixels(16, 16);
    let out = encode_png8(
        ImageRgb8 {
            rgb: &rgb,
            width: 16,
            height: 16,
        },
        PngCompression::Fast,
        OutputColorSpace::DisplayP3,
        None,
    )
    .unwrap();
    if !out.windows(4).any(|w| w == b"iCCP") {
        panic!("png missing iCCP chunk");
    }
}

#[test]
fn tiff_embeds_icc() {
    let rgb = red_pixels(16, 16);
    let out = encode_tiff8(
        ImageRgb8 {
            rgb: &rgb,
            width: 16,
            height: 16,
        },
        TiffCompression::None,
        OutputColorSpace::SRgb,
        None,
    )
    .unwrap();
    if !contains_icc(&out) {
        panic!("tiff missing ICC");
    }
}

#[test]
fn webp_embeds_icc() {
    let rgb = red_pixels(16, 16);
    let out = encode_webp_rgb(
        ImageRgb8 {
            rgb: &rgb,
            width: 16,
            height: 16,
        },
        85,
        false,
        OutputColorSpace::SRgb,
        None,
    )
    .unwrap();
    if &out[0..4] != b"RIFF" || &out[8..12] != b"WEBP" || &out[12..16] != b"VP8X" {
        panic!("expected RIFF/WEBP/VP8X header");
    }
    if !out.windows(4).any(|w| w == b"ICCP") {
        panic!("missing ICCP chunk");
    }
    if !contains_icc(&out) {
        panic!("webp missing ICC body");
    }
}
