use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{
    BitDepth, OutputColorSpace, OutputFormat, PngCompression, RenderOptions,
};
use raw_pipeline::{cpu, decode, encode};

const WIDTH: u32 = 256;
const HEIGHT: u32 = 16;
const STEP_PX: u32 = 32;

fn sky_gradient_png() -> Vec<u8> {
    let mut rgb = Vec::with_capacity((WIDTH * HEIGHT * 3) as usize);
    for _ in 0..HEIGHT {
        for x in 0..WIDTH {
            let v = 120 + (x / STEP_PX) as u8;
            rgb.extend_from_slice(&[v, v, v]);
        }
    }
    encode::encode_png8(
        encode::ImageRgb8 {
            rgb: &rgb,
            width: WIDTH,
            height: HEIGHT,
        },
        PngCompression::Fast,
        OutputColorSpace::SRgb,
    )
    .unwrap()
}

fn stretched_row() -> Vec<u16> {
    let frame = decode::decode(&sky_gradient_png()).unwrap();
    let mut edits = Edits::default();
    edits.basic.contrast = 100.0;
    let opts = RenderOptions {
        max_edge: WIDTH,
        output: OutputFormat::Png {
            bit_depth: BitDepth::Sixteen,
            compression: PngCompression::Fast,
        },
        ..Default::default()
    };
    let out = cpu::render(&frame, &edits, &opts).unwrap();
    let decoded = decode::decode(&out.bytes).unwrap();
    decoded
        .data
        .chunks_exact(3)
        .take(WIDTH as usize)
        .map(|px| (px[0].clamp(0.0, 1.0) * 65535.0).round() as u16)
        .collect()
}

#[test]
fn eight_bit_input_does_not_contour_under_a_steep_stretch() {
    let row = stretched_row();
    let source_levels = (WIDTH / STEP_PX) as usize;
    let mut levels: Vec<u16> = row.clone();
    levels.sort_unstable();
    levels.dedup();
    if levels.len() < source_levels * 8 {
        panic!(
            "stretched row holds {} levels for {source_levels} source levels; input dither is not breaking the steps",
            levels.len()
        );
    }
    let head = row[..STEP_PX as usize]
        .iter()
        .map(|v| *v as u32)
        .sum::<u32>();
    let tail = row[row.len() - STEP_PX as usize..]
        .iter()
        .map(|v| *v as u32)
        .sum::<u32>();
    if head >= tail {
        panic!("gradient direction was lost: head {head}, tail {tail}");
    }
}
