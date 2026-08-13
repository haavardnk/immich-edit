use super::*;

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
