use super::bitmap::{InputFormat, frame_from_rgb8, sniff_format};
use crate::math::srgb_to_linear;

#[test]
fn sniff_known_magics() {
    let cases: &[(&[u8], InputFormat)] = &[
        (&[0xFF, 0xD8, 0xFF, 0xE0], InputFormat::Jpeg),
        (b"\x89PNG\r\n\x1a\n", InputFormat::Png),
        (b"II*\0", InputFormat::Tiff),
        (b"MM\0*", InputFormat::Tiff),
        (b"RIFF\0\0\0\0WEBP", InputFormat::Webp),
        (b"\0\0\0\x20ftypheic", InputFormat::Heif),
        (b"\0\0\0\x20ftypmif1", InputFormat::Heif),
        (b"\0\0\0\x20ftypavif", InputFormat::Heif),
        (&[0xFF, 0x0A], InputFormat::Jxl),
        (b"\0\0\0\x0CJXL \r\n\x87\n", InputFormat::Jxl),
        (b"GIF87a", InputFormat::Gif),
        (b"GIF89a", InputFormat::Gif),
        (b"BM\0\0", InputFormat::Bmp),
    ];
    for (bytes, expected) in cases {
        if sniff_format(bytes) != Some(*expected) {
            panic!(
                "sniff failed for {expected:?}: got {:?}",
                sniff_format(bytes)
            );
        }
    }
}

#[test]
fn sniff_unknown_returns_none() {
    if sniff_format(b"not-an-image").is_some() {
        panic!("unknown bytes should not sniff");
    }
}

#[test]
fn rgb8_decode_dithers_deterministically() {
    let flat = vec![128u8; 64 * 64 * 3];
    let first = frame_from_rgb8(flat.clone(), 64, 64, None);
    let second = frame_from_rgb8(flat, 64, 64, None);
    if first.data != second.data {
        panic!("dither must be deterministic across decodes");
    }
    let exact = srgb_to_linear(128.0 / 255.0);
    if first.data.iter().all(|v| *v == exact) {
        panic!("flat patch was left undithered");
    }
    let mean = first.data.iter().sum::<f32>() / first.data.len() as f32;
    let step = srgb_to_linear(129.0 / 255.0) - exact;
    if (mean - exact).abs() > step * 0.05 {
        panic!("dither shifted the mean by {} (step {step})", mean - exact);
    }
    let worst = first
        .data
        .iter()
        .map(|v| (v - exact).abs())
        .fold(0.0f32, f32::max);
    if worst > step {
        panic!("dither exceeded one source LSB: {worst} > {step}");
    }
}
