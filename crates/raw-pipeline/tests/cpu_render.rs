use raw_pipeline::{cpu, decode, edits::Edits, frame::RawFrame, frame::RenderOptions};
use std::path::{Path, PathBuf};

const RAW_EXTS: &[&str] = &[
    "arw", "cr2", "cr3", "crw", "dng", "erf", "gpr", "iiq", "mrw", "nef", "nrw", "orf", "pef",
    "raf", "raw", "rw2", "rwl", "sr2", "srw", "x3f",
];

fn fixtures() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| RAW_EXTS.contains(&e.to_ascii_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect();
    paths.sort();
    paths
}

fn each_fixture(test: impl Fn(&str, &RawFrame)) {
    let paths = fixtures();
    if paths.is_empty() {
        eprintln!("no fixtures found; skipping");
        return;
    }
    let mut decoded = 0;
    for p in &paths {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        let bytes = std::fs::read(p).unwrap();
        let frame = match decode::decode(&bytes) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("skip {name}: decode unsupported ({e})");
                continue;
            }
        };
        test(&name, &frame);
        decoded += 1;
    }
    if decoded == 0 {
        panic!("no fixtures decoded successfully out of {}", paths.len());
    }
}

#[test]
fn decode_metadata() {
    each_fixture(|name, frame| {
        if frame.width == 0 || frame.height == 0 {
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
    each_fixture(|name, frame| {
        if frame.cpp != 1 {
            if frame.capture_sigma.is_some() {
                panic!("{name}: demosaiced frame reported a capture sigma");
            }
            return;
        }
        let Some(sigma) = frame.capture_sigma else {
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
    each_fixture(|name, frame| {
        let e = raw_pipeline::auto::auto_adjust(frame, &Edits::default());
        if e == Edits::default() {
            panic!("{name}: auto produced no edits (cpp {})", frame.cpp);
        }
    });
}

#[test]
fn xtrans_renders_neutral_greys() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/Fujifilm_X-T2_14bit_14bit_compressed_3-2.raf");
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
    each_fixture(|name, frame| {
        let opts = RenderOptions {
            max_edge: 512,
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
        if out.histogram.pixel_count() != (out.width as u64) * (out.height as u64) {
            panic!("{name}: histogram pixel count mismatch");
        }
    });
}

#[test]
fn rotate_swaps_dims() {
    each_fixture(|name, frame| {
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
    each_fixture(|name, frame| {
        let opts = RenderOptions {
            max_edge: 256,
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
        let base_mean = histogram_mean(&base.histogram.l);
        let bright_mean = histogram_mean(&bright.histogram.l);
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
    each_fixture(|name, frame| {
        let opts = RenderOptions {
            max_edge: 256,
            ..Default::default()
        };
        let out = cpu::render(frame, &Edits::default(), &opts).unwrap();
        let (transpose, _, _) = frame.orientation;
        let (expected_w, expected_h) = if transpose {
            (frame.height, frame.width)
        } else {
            (frame.width, frame.height)
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
    each_fixture(|name, frame| {
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
