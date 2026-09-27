use raw_pipeline::PipelineResult;
use raw_pipeline::decode::decode;
use raw_pipeline::encode::{ImageRgb8, encode_avif_rgb, encode_heic_rgb};
use raw_pipeline::frame::OutputColorSpace;
use raw_pipeline_testkit::roundtrip::{SPLIT_TONE_SIZE, assert_split_tone, split_tone_rgb};

type EncodeFn = fn(ImageRgb8<'_>, u8, OutputColorSpace) -> PipelineResult<Vec<u8>>;

fn roundtrip(encode_fn: EncodeFn) {
    let rgb = split_tone_rgb();
    let encoded = encode_fn(
        ImageRgb8 {
            rgb: &rgb,
            width: SPLIT_TONE_SIZE,
            height: SPLIT_TONE_SIZE,
        },
        90,
        OutputColorSpace::SRgb,
    )
    .expect("encode failed; the libheif codec plugin is missing");

    let frame = decode(&encoded).expect("decode failed; the libheif codec plugin is missing");
    assert_split_tone(&frame);
}

#[test]
fn heic_roundtrip_preserves_tones() {
    roundtrip(encode_heic_rgb);
}

#[test]
fn avif_roundtrip_preserves_tones() {
    roundtrip(encode_avif_rgb);
}
