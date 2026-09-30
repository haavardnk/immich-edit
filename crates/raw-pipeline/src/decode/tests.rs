use super::bitmap::{InputFormat, frame_from_rgb8, sniff_format};
use super::raw::scaled_mosaic;
use crate::math::srgb_to_linear;
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
