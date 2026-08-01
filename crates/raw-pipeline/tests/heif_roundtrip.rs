use raw_pipeline::PipelineResult;
use raw_pipeline::decode::decode;
use raw_pipeline::encode::{ImageRgb8, encode_avif_rgb, encode_heic_rgb};
use raw_pipeline::frame::{OutputColorSpace, RawFrame};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 64;

type EncodeFn = fn(ImageRgb8<'_>, u8, OutputColorSpace) -> PipelineResult<Vec<u8>>;

fn split_tone_rgb() -> Vec<u8> {
    (0..HEIGHT)
        .flat_map(|_| 0..WIDTH)
        .flat_map(|x| {
            let v = if x < WIDTH / 2 { 0u8 } else { 255u8 };
            [v, v, v]
        })
        .collect()
}

fn region_mean(frame: &RawFrame, x0: usize, x1: usize) -> f32 {
    let samples: Vec<f32> = (0..frame.height)
        .flat_map(|y| (x0..x1).map(move |x| (y * frame.width + x) * 3))
        .flat_map(|i| frame.data[i..i + 3].iter().copied())
        .collect();
    samples.iter().sum::<f32>() / samples.len() as f32
}

fn roundtrip(encode_fn: EncodeFn) {
    let rgb = split_tone_rgb();
    let encoded = encode_fn(
        ImageRgb8 {
            rgb: &rgb,
            width: WIDTH,
            height: HEIGHT,
        },
        90,
        OutputColorSpace::SRgb,
    )
    .expect("encode failed; the libheif codec plugin is missing");

    let frame = decode(&encoded).expect("decode failed; the libheif codec plugin is missing");

    assert_eq!(frame.width, WIDTH as usize);
    assert_eq!(frame.height, HEIGHT as usize);
    assert_eq!(frame.cpp, 3);
    assert!(!frame.is_raw);

    let dark = region_mean(&frame, 4, 28);
    let bright = region_mean(&frame, 36, 60);
    assert!(dark < 0.05, "dark half not preserved: {dark}");
    assert!(bright > 0.85, "bright half not preserved: {bright}");
}

#[test]
fn heic_roundtrip_preserves_tones() {
    roundtrip(encode_heic_rgb);
}

#[test]
fn avif_roundtrip_preserves_tones() {
    roundtrip(encode_avif_rgb);
}
