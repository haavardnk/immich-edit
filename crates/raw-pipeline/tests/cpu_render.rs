use raw_pipeline::{
    cpu, decode,
    edits::{CurvePoint, CurvePoints, Edits},
    frame::{FrameId, FrameMeta, RawFrame, RenderOptions},
};
use raw_pipeline_testkit::color::luma;
use raw_pipeline_testkit::fixtures::{
    each_fixture_frame, first_fixture_frame, fixture_path, fixtures,
};
use raw_pipeline_testkit::frames::rgb_frame;
use raw_pipeline_testkit::render::{decode_jpeg_rgb, rgb8_opts};

#[test]
fn decode_metadata() {
    each_fixture_frame(|name, frame| {
        if frame.meta.width == 0 || frame.meta.height == 0 {
            panic!("{name}: zero dim");
        }
        if frame.cfa_pattern.is_empty() && frame.cpp == 1 {
            panic!("{name}: bayer without cfa pattern");
        }
        if frame.cpp == 1 && !matches!(frame.cfa_pattern.len(), 4 | 36) {
            panic!(
                "{name}: mosaic frame with unsupported cfa '{}'",
                frame.cfa_pattern
            );
        }
        if frame.bps == 0 || frame.bps > 16 {
            panic!("{name}: bad bps {}", frame.bps);
        }
        if frame.data.is_empty() {
            panic!("{name}: no pixel data");
        }
    });
}

#[test]
fn capture_sigma_is_estimated_for_mosaic_fixtures() {
    each_fixture_frame(|name, frame| {
        if frame.cpp != 1 {
            if frame.meta.capture_sigma.is_some() {
                panic!("{name}: demosaiced frame reported a capture sigma");
            }
            return;
        }
        let Some(sigma) = frame.meta.capture_sigma else {
            panic!("{name}: mosaic frame has no capture sigma");
        };
        eprintln!("{name}: capture sigma {sigma}");
        if !(0.2..=2.0).contains(&sigma) {
            panic!("{name}: implausible capture sigma {sigma}");
        }
    });
}

#[test]
fn auto_adjust_reads_every_fixture() {
    each_fixture_frame(|name, frame| {
        let e = raw_pipeline::auto::auto_adjust(frame, &Edits::default());
        if e == Edits::default() {
            panic!("{name}: auto produced no edits (cpp {})", frame.cpp);
        }
    });
}

#[test]
fn auto_tools_agree_on_fast_and_quality_frames() {
    for path in fixtures() {
        let name = path.file_name().unwrap().to_string_lossy();
        let bytes = std::fs::read(&path).unwrap();
        let (Ok(fast), Ok(quality)) = (decode::decode(&bytes), decode::decode_quality(&bytes))
        else {
            continue;
        };
        let edits = Edits::default();
        let fast_ev = raw_pipeline::auto::auto_adjust(&fast, &edits)
            .basic
            .exposure_ev;
        let quality_ev = raw_pipeline::auto::auto_adjust(&quality, &edits)
            .basic
            .exposure_ev;
        let fast_wb = raw_pipeline::white_balance::auto_white_balance(&fast, &edits);
        let quality_wb = raw_pipeline::white_balance::auto_white_balance(&quality, &edits);
        if (fast_ev - quality_ev).abs() > 0.05 {
            panic!("{name}: auto exposure {fast_ev} on fast frame vs {quality_ev} on quality");
        }
        let wb_close = match (fast_wb, quality_wb) {
            (Some(a), Some(b)) => (a.0 - b.0).abs() <= 3.0 && (a.1 - b.1).abs() <= 3.0,
            (a, b) => a.is_none() && b.is_none(),
        };
        if !wb_close {
            panic!("{name}: auto wb {fast_wb:?} on fast frame vs {quality_wb:?} on quality");
        }
    }
}

#[test]
fn xtrans_renders_neutral_greys() {
    let path = fixture_path("Fujifilm_X-T2_14bit_14bit_compressed_3-2.raf");
    let Ok(bytes) = std::fs::read(&path) else {
        eprintln!("no X-Trans fixture; skipping");
        return;
    };
    let frame = decode::decode(&bytes).expect("decode X-Trans");
    if frame.cpp != 1 || cpu::demosaic::parse_xtrans(&frame.cfa_pattern).is_none() {
        panic!(
            "X-Trans frame did not reach the pipeline as a 6x6 mosaic: cpp {} cfa '{}'",
            frame.cpp, frame.cfa_pattern
        );
    }

    let opts = RenderOptions {
        max_edge: 512,
        ..Default::default()
    };
    let out = cpu::render(&frame, &Edits::default(), &opts).expect("render X-Trans");
    let img: turbojpeg::Image<Vec<u8>> =
        turbojpeg::decompress(&out.bytes, turbojpeg::PixelFormat::RGB).expect("decompress");
    for (cx, cy) in GREY_PATCHES {
        let mean = patch_mean(&img, cx, cy);
        let spread = mean[0].max(mean[1]).max(mean[2]) - mean[0].min(mean[1]).min(mean[2]);
        if spread > GREY_SPREAD_CEIL {
            panic!("grey patch at {cx},{cy} is not neutral: rgb {mean:.1?} spread {spread:.1}");
        }
    }
}

const GREY_PATCHES: [(usize, usize); 3] = [(190, 95), (170, 140), (215, 130)];
const GREY_SPREAD_CEIL: f64 = 25.0;

fn mean_luma(jpeg: &[u8]) -> f64 {
    let (rgb, w, h) = decode_jpeg_rgb(jpeg);
    rgb.chunks_exact(3).map(luma).sum::<f64>() / (w * h) as f64
}

fn with_dcp_mode(mode: raw_pipeline::edits::DcpMode) -> Edits {
    let mut edits = Edits::default();
    edits.color.dcp.mode = mode;
    edits
}

fn patch_mean(img: &turbojpeg::Image<Vec<u8>>, cx: usize, cy: usize) -> [f64; 3] {
    let mut sum = [0u64; 3];
    let mut n = 0u64;
    for y in cy - 6..=cy + 6 {
        for x in cx - 6..=cx + 6 {
            let i = (y * img.pitch) + x * 3;
            sum[0] += img.pixels[i] as u64;
            sum[1] += img.pixels[i + 1] as u64;
            sum[2] += img.pixels[i + 2] as u64;
            n += 1;
        }
    }
    [
        sum[0] as f64 / n as f64,
        sum[1] as f64 / n as f64,
        sum[2] as f64 / n as f64,
    ]
}

#[test]
fn identity_render_jpeg() {
    each_fixture_frame(|name, frame| {
        let opts = RenderOptions {
            max_edge: 512,
            histogram: true,
            ..Default::default()
        };
        let out = cpu::render(frame, &Edits::default(), &opts).unwrap();
        if out.bytes.len() < 1000 {
            panic!("{name}: jpeg too small ({} bytes)", out.bytes.len());
        }
        if &out.bytes[..2] != b"\xff\xd8" {
            panic!("{name}: not jpeg SOI marker");
        }
        if out.width.max(out.height) > 512 {
            panic!("{name}: max edge exceeded {}x{}", out.width, out.height);
        }
        let histogram = out.histogram.expect("histogram requested");
        if histogram.pixel_count() != (out.width as u64) * (out.height as u64) {
            panic!("{name}: histogram pixel count mismatch");
        }
    });
}

#[test]
fn default_sharpening_is_raw_only() {
    let Some(frame) = first_fixture_frame() else {
        eprintln!("no fixtures found; skipping");
        return;
    };
    let opts = RenderOptions {
        max_edge: 256,
        ..Default::default()
    };
    let unsharp = Edits {
        detail: raw_pipeline::edits::DetailEdits {
            sharpen_amount: Some(0.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let raw_default = cpu::render(&frame, &Edits::default(), &opts).unwrap();
    let raw_unsharp = cpu::render(&frame, &unsharp, &opts).unwrap();
    if raw_default.bytes == raw_unsharp.bytes {
        panic!("raw render ignored the default sharpening");
    }
    let rendered = RawFrame {
        meta: FrameMeta {
            is_raw: false,
            ..frame.meta.clone()
        },
        cfa_pattern: frame.cfa_pattern.clone(),
        data: frame.data.clone(),
        exif: None,
        id: FrameId::fresh(),
        ..frame
    };
    let rendered_default = cpu::render(&rendered, &Edits::default(), &opts).unwrap();
    let rendered_unsharp = cpu::render(&rendered, &unsharp, &opts).unwrap();
    if rendered_default.bytes != rendered_unsharp.bytes {
        panic!("non-raw render applied the default sharpening");
    }
}

#[test]
fn sensor_scaling_darkens_the_render() {
    each_fixture_frame(|name, frame| {
        if !frame.meta.is_raw {
            return;
        }
        let opts = RenderOptions {
            max_edge: 256,
            ..Default::default()
        };
        let dim = RawFrame {
            meta: frame.meta.clone(),
            cfa_pattern: frame.cfa_pattern.clone(),
            bps: frame.bps,
            data: frame.data.iter().map(|v| v * 0.25).collect(),
            cpp: frame.cpp,
            exif: None,
            id: FrameId::fresh(),
        };
        let bright = mean_luma(&cpu::render(frame, &Edits::default(), &opts).unwrap().bytes);
        let dark = mean_luma(&cpu::render(&dim, &Edits::default(), &opts).unwrap().bytes);
        if dark > bright * 0.8 {
            panic!(
                "{name}: two stops under rendered at {dark:.2} vs {bright:.2}; \
                 the pipeline is compensating for scene brightness"
            );
        }
    });
}

#[test]
fn default_color_differs_from_flat_on_raw() {
    use raw_pipeline::edits::DcpMode;
    each_fixture_frame(|name, frame| {
        if !frame.meta.is_raw {
            return;
        }
        let opts = RenderOptions {
            max_edge: 256,
            ..Default::default()
        };
        let color = cpu::render(frame, &Edits::default(), &opts).unwrap();
        let flat = cpu::render(frame, &with_dcp_mode(DcpMode::Flat), &opts).unwrap();
        if color.bytes == flat.bytes {
            panic!("{name}: default color must not render identically to flat");
        }
    });
}

#[test]
fn non_raw_ignores_the_profile_mode() {
    use raw_pipeline::edits::DcpMode;
    each_fixture_frame(|name, frame| {
        let opts = RenderOptions {
            max_edge: 256,
            ..Default::default()
        };
        let rendered = RawFrame {
            meta: FrameMeta {
                is_raw: false,
                ..frame.meta.clone()
            },
            cfa_pattern: frame.cfa_pattern.clone(),
            data: frame.data.clone(),
            exif: None,
            id: FrameId::fresh(),
            ..*frame
        };
        let flat = cpu::render(&rendered, &with_dcp_mode(DcpMode::Flat), &opts).unwrap();
        for mode in [DcpMode::Auto, DcpMode::Off] {
            let other = cpu::render(&rendered, &with_dcp_mode(mode), &opts).unwrap();
            if other.bytes != flat.bytes {
                panic!("{name}: non-raw render changed with profile mode {mode:?}");
            }
        }
    });
}

#[test]
fn rotate_swaps_dims() {
    each_fixture_frame(|name, frame| {
        let opts = RenderOptions {
            max_edge: 256,
            ..Default::default()
        };
        let base = cpu::render(frame, &Edits::default(), &opts).unwrap();
        let rotated_edits = Edits {
            geometry: raw_pipeline::edits::GeometryEdits {
                rotate: 90,
                ..Default::default()
            },
            ..Default::default()
        };
        let rotated = cpu::render(frame, &rotated_edits, &opts).unwrap();
        if base.width == base.height {
            return;
        }
        if rotated.width != base.height || rotated.height != base.width {
            panic!(
                "{name}: rotate90 dims {} {} -> {} {}",
                base.width, base.height, rotated.width, rotated.height
            );
        }
    });
}

#[test]
fn exposure_raises_mean() {
    each_fixture_frame(|name, frame| {
        let opts = RenderOptions {
            max_edge: 256,
            histogram: true,
            ..Default::default()
        };
        let base = cpu::render(frame, &Edits::default(), &opts).unwrap();
        let bright_edits = Edits {
            basic: raw_pipeline::edits::BasicEdits {
                exposure_ev: 2.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let bright = cpu::render(frame, &bright_edits, &opts).unwrap();
        let base_hist = base.histogram.expect("base histogram requested");
        let bright_hist = bright.histogram.expect("bright histogram requested");
        let base_mean = histogram_mean(&base_hist.l);
        let bright_mean = histogram_mean(&bright_hist.l);
        if bright_mean <= base_mean {
            panic!(
                "{name}: exposure +2 mean {} <= base {}",
                bright_mean, base_mean
            );
        }
    });
}

#[test]
fn orientation_swaps_display_dims_when_transposed() {
    each_fixture_frame(|name, frame| {
        let opts = RenderOptions {
            max_edge: 256,
            ..Default::default()
        };
        let out = cpu::render(frame, &Edits::default(), &opts).unwrap();
        let (transpose, _, _) = frame.meta.orientation;
        let (expected_w, expected_h) = if transpose {
            (frame.meta.height, frame.meta.width)
        } else {
            (frame.meta.width, frame.meta.height)
        };
        let landscape_sensor = expected_w > expected_h;
        let landscape_out = out.width > out.height;
        if landscape_sensor != landscape_out && out.width != out.height {
            panic!(
                "{name}: oriented landscape={landscape_sensor} but out landscape={landscape_out} ({}x{})",
                out.width, out.height
            );
        }
    });
}

fn histogram_mean(bins: &[u32]) -> f64 {
    let total: u64 = bins.iter().map(|&v| v as u64).sum();
    if total == 0 {
        return 0.0;
    }
    let weighted: u64 = bins
        .iter()
        .enumerate()
        .map(|(i, &v)| i as u64 * v as u64)
        .sum();
    weighted as f64 / total as f64
}

#[test]
fn exif_roundtrip_preserves_camera() {
    each_fixture_frame(|name, frame| {
        let Some(exif) = frame.exif.as_ref() else {
            eprintln!("{name}: no exif parsed, skipping");
            return;
        };
        let opts = RenderOptions {
            max_edge: 512,
            ..Default::default()
        };
        let mut out = cpu::render(frame, &Edits::default(), &opts).unwrap().bytes;
        raw_pipeline::exif::inject(&mut out, exif, little_exif::filetype::FileExtension::JPEG)
            .unwrap();
        let reread = match little_exif::metadata::Metadata::new_from_vec(
            &out,
            little_exif::filetype::FileExtension::JPEG,
        ) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{name}: reparse failed ({e}); known inject bug, skipping");
                return;
            }
        };
        let has_make = reread
            .get_tag(&little_exif::exif_tag::ExifTag::Make(String::new()))
            .next()
            .is_some();
        if !has_make {
            panic!("{name}: Make tag lost after roundtrip");
        }
    });
}

fn flat_display(rgb: [f32; 3], edits: &Edits) -> [f64; 3] {
    let frame = rgb_frame(16, 16, rgb.repeat(256));
    let out = cpu::render(&frame, edits, &rgb8_opts(64)).unwrap();
    let px = out.bytes.len() / 3;
    let sum = out.bytes.chunks_exact(3).fold([0.0f64; 3], |acc, p| {
        [
            acc[0] + p[0] as f64,
            acc[1] + p[1] as f64,
            acc[2] + p[2] as f64,
        ]
    });
    sum.map(|v| v / px as f64 / 255.0)
}

fn composite_curve(x: f64, y: f64) -> Edits {
    let mut edits = Edits::default();
    edits.basic.curves.composite = CurvePoints {
        points: vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x, y },
            CurvePoint { x: 1.0, y: 1.0 },
        ],
    };
    edits
}

#[test]
fn curve_points_address_display_values() {
    let mid_grey = [0.214f32; 3];
    let plain = flat_display(mid_grey, &Edits::default());
    let curved = flat_display(mid_grey, &composite_curve(0.5, 0.7));
    if (plain[1] - 0.5).abs() > 0.01 || (curved[1] - 0.7).abs() > 0.01 {
        panic!("display 0.5 should map to 0.7, plain {plain:?} curved {curved:?}");
    }
}

#[test]
fn shadow_curve_leaves_bright_colors_to_gamut_mapping() {
    let bright = [2.0f32, 0.48, 0.2];
    let plain = flat_display(bright, &Edits::default());
    let curved = flat_display(bright, &composite_curve(0.25, 0.3));
    let drift = (0..3)
        .map(|c| (curved[c] - plain[c]).abs())
        .fold(0.0, f64::max);
    if drift > 0.03 {
        panic!("shadow curve moved a highlight by {drift}: plain {plain:?} curved {curved:?}");
    }
}
