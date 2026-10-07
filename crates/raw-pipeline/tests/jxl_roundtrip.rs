use raw_pipeline::decode::decode;
use raw_pipeline::encode::{ImageRgb8, encode_jxl8, encode_jxl16};
use raw_pipeline::frame::OutputColorSpace;
use raw_pipeline_testkit::roundtrip::{SPLIT_TONE_SIZE, assert_split_tone, split_tone_rgb};

fn encode8(cs: OutputColorSpace) -> Vec<u8> {
    let rgb = split_tone_rgb();
    encode_jxl8(
        ImageRgb8 {
            rgb: &rgb,
            width: SPLIT_TONE_SIZE,
            height: SPLIT_TONE_SIZE,
        },
        cs,
        None,
    )
    .expect("jxl 8-bit encode failed")
}

fn encode16(cs: OutputColorSpace) -> Vec<u8> {
    let rgb16: Vec<u16> = split_tone_rgb()
        .into_iter()
        .map(|v| u16::from(v) * 257)
        .collect();
    encode_jxl16(&rgb16, SPLIT_TONE_SIZE, SPLIT_TONE_SIZE, cs, None)
        .expect("jxl 16-bit encode failed")
}

fn assert_tones_survive(encoded: &[u8]) {
    assert_split_tone(&decode(encoded).expect("jxl decode failed"));
}

#[test]
fn jxl8_srgb_roundtrip_preserves_tones() {
    assert_tones_survive(&encode8(OutputColorSpace::SRgb));
}

#[test]
fn jxl8_display_p3_roundtrip_preserves_tones() {
    assert_tones_survive(&encode8(OutputColorSpace::DisplayP3));
}

#[test]
fn jxl16_srgb_roundtrip_preserves_tones() {
    assert_tones_survive(&encode16(OutputColorSpace::SRgb));
}

#[test]
fn jxl_display_p3_writes_distinct_color_encoding() {
    assert_ne!(
        encode8(OutputColorSpace::SRgb),
        encode8(OutputColorSpace::DisplayP3)
    );
}
