use super::Develop;
use super::bitmap::{Bitmap, InputFormat, sniff_format};
use super::raw::scaled_mosaic;
use crate::PipelineError;
use crate::encode::{ImageRgb8, encode_png8};
use crate::frame::{OutputColorSpace, PngCompression};
use crate::math::srgb_to_linear;
use raw_pipeline_testkit::fixtures::fixture_path;
use rawler::cfa::CFA;
use rawler::imgop::develop::{Intermediate, ProcessingStep, RawDevelop};

const XTRANS: &str = "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG";

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
fn scaled_mosaic_matches_rawler_rescale_and_crop() {
    let params = rawler::decoders::RawDecodeParams::default();
    let mut checked = 0;
    for file in raw_pipeline_testkit::fixtures::fixtures() {
        let Ok(bytes) = std::fs::read(&file) else {
            continue;
        };
        let source = rawler::rawsource::RawSource::new_from_slice(&bytes);
        let Ok(raw) = rawler::decode(&source, &params) else {
            continue;
        };
        if raw.cpp != 1 {
            continue;
        }
        let develop = RawDevelop {
            steps: vec![ProcessingStep::Rescale],
        };
        let Ok(Intermediate::Monochrome(pixels)) = develop.develop_intermediate(&raw) else {
            panic!("{} did not develop to a mosaic", file.display());
        };
        let expected = match raw.active_area {
            Some(area) => pixels.crop(area),
            None => pixels,
        };
        let (data, width, height) = scaled_mosaic(&raw);
        if (width, height) != (expected.width, expected.height) {
            panic!("{} dims differ", file.display());
        }
        let (x0, y0) = raw.active_area.map_or((0, 0), |area| (area.p.x, area.p.y));
        let rawler_scaled =
            |i: usize| x0 + i % width < raw.width & !1 && y0 + i / width < raw.height & !1;
        let mismatch = data
            .iter()
            .zip(expected.into_inner())
            .enumerate()
            .any(|(i, (ours, theirs))| rawler_scaled(i) && *ours != theirs);
        if mismatch {
            panic!("{} pixels differ from rawler", file.display());
        }
        checked += 1;
    }
    if checked < 5 {
        panic!("only {checked} CFA fixtures checked");
    }
}

#[test]
fn png_decodes_every_colour_type() {
    let palette: &[u8] = &[50, 50, 50, 255, 255, 255];
    let cases: [(png::ColorType, png::BitDepth, &[u8]); 4] = [
        (
            png::ColorType::GrayscaleAlpha,
            png::BitDepth::Eight,
            &[50, 255, 255, 0],
        ),
        (
            png::ColorType::GrayscaleAlpha,
            png::BitDepth::Sixteen,
            &[50, 0, 255, 255, 255, 255, 0, 0],
        ),
        (
            png::ColorType::Grayscale,
            png::BitDepth::One,
            &[0b0100_0000],
        ),
        (png::ColorType::Indexed, png::BitDepth::Eight, &[0, 1]),
    ];
    for (color, depth, pixels) in cases {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, 2, 1);
        encoder.set_color(color);
        encoder.set_depth(depth);
        if color == png::ColorType::Indexed {
            encoder.set_palette(palette);
        }
        encoder
            .write_header()
            .unwrap()
            .write_image_data(pixels)
            .unwrap();
        let frame = match super::decode_image(&bytes, None, None) {
            Ok(frame) => frame,
            Err(e) => panic!("{color:?} {depth:?}: {e}"),
        };
        let grey = frame.data.len() == 6
            && frame
                .data
                .chunks_exact(3)
                .all(|p| (p[0] - p[1]).abs() < 0.01 && (p[0] - p[2]).abs() < 0.01);
        if !grey || frame.data[3] - frame.data[0] < 0.3 {
            panic!(
                "{color:?} {depth:?}: not a dark-to-bright 2x1 grey frame: {:?}",
                frame.data
            );
        }
    }
}

#[test]
fn rgb8_decode_dithers_deterministically() {
    let flat = vec![128u8; 64 * 64 * 3];
    let first = Bitmap::rgb8(flat.clone(), 64, 64).into_frame(None);
    let second = Bitmap::rgb8(flat, 64, 64).into_frame(None);
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

#[test]
fn cfa_shift_offsets_by_x_then_y() {
    let cases = [XTRANS, "RGGB", "BGGR"]
        .into_iter()
        .flat_map(|name| [(0, 0), (1, 0), (0, 1), (2, 3), (7, 11)].map(|(dx, dy)| (name, dx, dy)));
    for (name, dx, dy) in cases {
        let cfa = CFA::new(name);
        let shifted = cfa.shift(dx, dy);
        let dim = cfa.width;
        let wrong = (0..dim * dim).find(|&i| {
            let x = i % dim;
            let y = i / dim;
            shifted.name.as_bytes()[i]
                != cfa.name.as_bytes()[((y + dy) % dim) * dim + (x + dx) % dim]
        });
        if let Some(i) = wrong {
            panic!(
                "{name} shifted by {dx},{dy} is '{}', wrong at index {i}",
                shifted.name
            );
        }
    }
}

fn admits_exactly(name: &str, data: &[u8], develop: Develop) {
    let frame = match super::decode_within(data, develop, None) {
        Ok(frame) => frame,
        Err(e) => panic!("{name} {develop:?}: {e}"),
    };
    let frame_bytes = size_of_val(frame.data.as_slice()) as u64;
    match super::decode_within(data, develop, Some(frame_bytes - 1)) {
        Err(PipelineError::OverBudget(message)) if message.contains("decoding this photo") => {}
        Err(e) => panic!("{name} {develop:?}: refused with the wrong error: {e}"),
        Ok(_) => panic!("{name} {develop:?}: a frame over the budget was decoded"),
    }
    if let Err(e) = super::decode_within(data, develop, Some(frame_bytes)) {
        panic!("{name} {develop:?}: a frame at the budget was refused: {e}");
    }
}

#[test]
fn a_frame_over_the_memory_budget_is_refused_before_it_is_built() {
    for name in [
        "Canon_EOS_5D_3-2.cr2",
        "Fujifilm_X-T2_14bit_14bit_compressed_3-2.raf",
    ] {
        let bytes = std::fs::read(fixture_path(name)).unwrap();
        for develop in [Develop::Fast, Develop::Quality] {
            admits_exactly(name, &bytes, develop);
        }
    }
    let rgb = vec![100u8; 64 * 48 * 3];
    let image = ImageRgb8 {
        rgb: &rgb,
        width: 64,
        height: 48,
    };
    let png = encode_png8(image, PngCompression::Fast, OutputColorSpace::SRgb, None).unwrap();
    admits_exactly("png", &png, Develop::Fast);
}
