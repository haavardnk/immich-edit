use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use little_exif::exif_tag::ExifTag;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{
    BitDepth, JpegSubsampling, OutputColorSpace, OutputFormat, PngCompression, RawFrame,
    RenderOptions, TiffCompression,
};
use raw_pipeline::metadata::ExportMetadata;
use raw_pipeline::{cpu, decode, encode};
use raw_pipeline_testkit::fixtures::{fixtures, image_fixtures};
use raw_pipeline_testkit::frames::synthetic_frame;
use serde_json::{Map, Value};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 48;
const MIN_EXIFTOOL: f64 = 12.5;
const MAX_TAG_BYTES: u64 = 4096;

const FORMATS: [(&str, OutputFormat, OutputColorSpace); 7] = [
    (
        "jpg",
        OutputFormat::Jpeg {
            quality: 85,
            subsampling: JpegSubsampling::Chroma420,
        },
        OutputColorSpace::SRgb,
    ),
    (
        "png",
        OutputFormat::Png {
            bit_depth: BitDepth::Eight,
            compression: PngCompression::Fast,
        },
        OutputColorSpace::DisplayP3,
    ),
    (
        "tif",
        OutputFormat::Tiff {
            bit_depth: BitDepth::Sixteen,
            compression: TiffCompression::Deflate,
        },
        OutputColorSpace::SRgb,
    ),
    (
        "webp",
        OutputFormat::Webp {
            quality: 80,
            lossless: false,
        },
        OutputColorSpace::DisplayP3,
    ),
    (
        "avif",
        OutputFormat::Avif { quality: 60 },
        OutputColorSpace::SRgb,
    ),
    (
        "heic",
        OutputFormat::Heic { quality: 60 },
        OutputColorSpace::DisplayP3,
    ),
    (
        "jxl",
        OutputFormat::Jxl {
            bit_depth: BitDepth::Eight,
        },
        OutputColorSpace::SRgb,
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    All,
    NoLocation,
    None,
}

const MODES: [Mode; 3] = [Mode::All, Mode::NoLocation, Mode::None];

const CAPTURE_TAGS: [&str; 11] = [
    "DateTimeOriginal",
    "OffsetTimeOriginal",
    "Make",
    "Model",
    "LensModel",
    "ExposureTime",
    "FNumber",
    "ISO",
    "FocalLength",
    "GPSLatitude",
    "GPSLongitude",
];

const LAYOUT_IDS: [(u32, u32); 22] = [
    (0x00FE, 0x0103),
    (0x0106, 0x010A),
    (0x0111, 0x0111),
    (0x0115, 0x0119),
    (0x011C, 0x011C),
    (0x0122, 0x0125),
    (0x012D, 0x012D),
    (0x013D, 0x0146),
    (0x014A, 0x014A),
    (0x014C, 0x014C),
    (0x0150, 0x0156),
    (0x015B, 0x015B),
    (0x0200, 0x0209),
    (0x0211, 0x0214),
    (0x02BC, 0x02BC),
    (0x83BB, 0x83BB),
    (0x8649, 0x8649),
    (0x8773, 0x8773),
    (0x927C, 0x927C),
    (0x935C, 0x935C),
    (0xA420, 0xA420),
    (0xC612, 0xC7FF),
];

const FORBIDDEN_IDS: [u32; 4] = [0x83BB, 0x02BC, 0x927C, 0xA420];
const DNG_PRIVATE: (u32, u32) = (0xC612, 0xC7FF);
const PANASONIC_RAW: (u32, u32) = (0x0001, 0x00FD);
const REWRITTEN_IDS: [u32; 5] = [0x0112, 0x0131, 0xA001, 0xA002, 0xA003];
const RESOLUTION_IDS: [u32; 3] = [0x011A, 0x011B, 0x0128];

const IDENTITY_TAGS: [&str; 8] = [
    "ContentIdentifier",
    "BurstUUID",
    "BurstID",
    "MotionPhoto",
    "MotionPhotoVersion",
    "MicroVideo",
    "MicroVideoOffset",
    "MediaGroupUUID",
];

struct Tag {
    key: String,
    family0: String,
    family1: String,
    name: String,
    id: Option<u32>,
    value: Value,
}

impl Tag {
    fn exif(&self) -> bool {
        self.family0 == "EXIF"
    }

    fn id_in(&self, ranges: &[(u32, u32)]) -> bool {
        self.exif()
            && self
                .id
                .is_some_and(|id| ranges.iter().any(|(lo, hi)| (*lo..=*hi).contains(&id)))
    }

    fn id_is(&self, ids: &[u32]) -> bool {
        self.exif() && self.id.is_some_and(|id| ids.contains(&id))
    }

    fn binary_bytes(&self) -> Option<u64> {
        let text = self.value.as_str()?.strip_prefix("(Binary data ")?;
        text.split(' ').next()?.parse().ok()
    }

    fn location(&self) -> bool {
        self.exif() && self.family1 == "GPS"
    }
}

struct Case {
    source: usize,
    format: usize,
    mode: Mode,
    path: PathBuf,
}

fn parse_tag(key: &str, entry: &Value) -> Option<Tag> {
    let parts: Vec<&str> = key.split(':').collect();
    let (family0, family1, name) = match parts.as_slice() {
        [f0, f1, name] => (*f0, *f1, *name),
        [f0, name] => (*f0, *f0, *name),
        _ => return None,
    };
    Some(Tag {
        key: key.to_string(),
        family0: family0.to_string(),
        family1: family1.to_string(),
        name: name.to_string(),
        id: entry.get("id")?.as_u64().map(|id| id as u32),
        value: entry.get("val")?.clone(),
    })
}

fn exiftool_version() -> f64 {
    let out = match Command::new("exiftool").arg("-ver").output() {
        Ok(out) => out,
        Err(e) => panic!("exiftool is required for this test (see docs/development.md): {e}"),
    };
    let text = String::from_utf8_lossy(&out.stdout);
    match text.trim().parse::<f64>() {
        Ok(v) => v,
        Err(_) => panic!("unexpected exiftool -ver output: {text:?}"),
    }
}

fn read_tags(dir: &Path, paths: &[&Path]) -> BTreeMap<PathBuf, Vec<Tag>> {
    let list = dir.join("exiftool-args.txt");
    let lines: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
    std::fs::write(&list, lines.join("\n")).unwrap();
    let out = Command::new("exiftool")
        .args([
            "-json",
            "-G0:1",
            "-n",
            "-D",
            "-a",
            "-charset",
            "filename=utf8",
            "-@",
        ])
        .arg(&list)
        .output()
        .unwrap();
    let parsed: Vec<Map<String, Value>> = match serde_json::from_slice(&out.stdout) {
        Ok(v) => v,
        Err(e) => panic!(
            "exiftool output is not JSON ({e}): {}",
            String::from_utf8_lossy(&out.stderr)
        ),
    };
    parsed
        .into_iter()
        .filter_map(|file| {
            let path = PathBuf::from(file.get("SourceFile")?.as_str()?);
            let tags = file
                .iter()
                .filter_map(|(key, entry)| parse_tag(key, entry))
                .collect();
            Some((path, tags))
        })
        .collect()
}

fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(x), Some(y)) => (x - y).abs() <= 1e-6 * x.abs().max(y.abs()).max(1.0),
            _ => x == y,
        },
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same(x, y))
        }
        _ => a == b,
    }
}

fn empty(value: &Value) -> bool {
    value.as_str().is_some_and(|s| s.trim().is_empty())
}

fn compared_exif(tag: &Tag) -> bool {
    tag.exif()
        && matches!(tag.family1.as_str(), "IFD0" | "ExifIFD" | "GPS")
        && !tag.id_in(&LAYOUT_IDS)
        && !(tag.family1 == "IFD0" && tag.id_in(&[PANASONIC_RAW]))
        && !tag.id_is(&REWRITTEN_IDS)
        && tag.binary_bytes().is_none_or(|n| n <= MAX_TAG_BYTES)
}

fn find(tags: &[Tag], f: impl Fn(&Tag) -> bool) -> Option<&Tag> {
    tags.iter().find(|t| f(t))
}

fn check_forbidden(export: &[Tag], mode: Mode, failures: &mut Vec<(&'static str, String)>) {
    for tag in export {
        let detail = format!("{} = {}", tag.key, tag.value);
        if IDENTITY_TAGS.contains(&tag.name.as_str()) {
            failures.push(("identity", detail.clone()));
        }
        let stale = matches!(
            tag.family0.as_str(),
            "MakerNotes" | "IPTC" | "Photoshop" | "XMP"
        ) || (tag.exif() && matches!(tag.family1.as_str(), "IFD1" | "InteropIFD"))
            || tag.id_is(&FORBIDDEN_IDS)
            || tag.id_in(&[DNG_PRIVATE]);
        if stale {
            failures.push(("stale", detail.clone()));
        }
        if tag.name == "Orientation"
            && tag.family0 != "Composite"
            && !same(&tag.value, &Value::from(1))
        {
            failures.push(("orientation", detail.clone()));
        }
        let structural = tag.id_in(&LAYOUT_IDS) || tag.id_is(&RESOLUTION_IDS);
        if mode == Mode::None && tag.exif() && !structural {
            failures.push(("none", detail.clone()));
        }
        if mode != Mode::All && tag.location() {
            failures.push(("location", detail));
        }
    }
}

fn check_capture(
    source: &[Tag],
    export: &[Tag],
    mode: Mode,
    failures: &mut Vec<(&'static str, String)>,
) {
    for name in CAPTURE_TAGS {
        if mode != Mode::All && name.starts_with("GPS") {
            continue;
        }
        let candidates: Vec<&Tag> = source
            .iter()
            .filter(|t| t.name == name && !empty(&t.value))
            .filter(|t| {
                !matches!(
                    t.family0.as_str(),
                    "Composite" | "XMP" | "File" | "ExifTool"
                )
            })
            .collect();
        if candidates.is_empty() {
            continue;
        }
        let expected: Vec<&Value> = candidates
            .iter()
            .filter(|t| t.family0 != "MakerNotes")
            .map(|t| &t.value)
            .collect();
        let got = find(export, |t| t.exif() && t.name == name).map(|t| &t.value);
        let matched = match got {
            Some(v) => expected.is_empty() || expected.iter().any(|e| same(e, v)),
            None => false,
        };
        if !matched {
            failures.push((
                "capture",
                format!("{name}: expected one of {expected:?}, got {got:?}"),
            ));
        }
    }
}

fn check_parity(
    source: &[Tag],
    export: &[Tag],
    mode: Mode,
    failures: &mut Vec<(&'static str, String)>,
) {
    for tag in source {
        if !compared_exif(tag) || empty(&tag.value) || (mode != Mode::All && tag.location()) {
            continue;
        }
        let got = find(export, |t| t.key == tag.key).map(|t| &t.value);
        if got.is_some_and(|v| same(&tag.value, v)) {
            continue;
        }
        failures.push((
            "exif-parity",
            format!("{}: expected {}, got {got:?}", tag.key, tag.value),
        ));
    }
}

fn check_rewritten(
    source: &[Tag],
    export: &[Tag],
    cs: OutputColorSpace,
    failures: &mut Vec<(&'static str, String)>,
) {
    if !source.iter().any(compared_exif) {
        return;
    }
    let color_space = match cs {
        OutputColorSpace::SRgb => 1,
        OutputColorSpace::DisplayP3 => 65535,
    };
    let expected = [
        ("ColorSpace", Value::from(color_space)),
        ("ExifImageWidth", Value::from(WIDTH)),
        ("ExifImageHeight", Value::from(HEIGHT)),
        (
            "Software",
            Value::from(format!("immich-edit {}", raw_pipeline::version())),
        ),
    ];
    for (name, value) in expected {
        let got = find(export, |t| t.exif() && t.name == name).map(|t| &t.value);
        if !got.is_some_and(|v| same(v, &value)) {
            failures.push((
                "rewritten",
                format!("{name}: expected {value}, got {got:?}"),
            ));
        }
    }
}

fn encoder_owned(tags: &[Tag]) -> Vec<String> {
    let mut owned: Vec<String> = tags
        .iter()
        .filter_map(|t| {
            if t.family0 == "ICC_Profile" && t.name == "ProfileDescription" {
                return Some(format!("{} = {}", t.key, t.value));
            }
            (t.family1 == "IFD0" && t.id_in(&LAYOUT_IDS)).then(|| t.key.clone())
        })
        .collect();
    owned.sort();
    owned
}

fn check_encoder_owned(export: &[Tag], plain: &[Tag], failures: &mut Vec<(&'static str, String)>) {
    let got = encoder_owned(export);
    let expected = encoder_owned(plain);
    if got != expected {
        failures.push((
            "encoder-owned",
            format!("expected {expected:?}, got {got:?}"),
        ));
    }
}

fn check(
    source: &[Tag],
    export: &[Tag],
    plain: &[Tag],
    mode: Mode,
    cs: OutputColorSpace,
) -> Vec<(&'static str, String)> {
    let mut failures = Vec::new();
    check_forbidden(export, mode, &mut failures);
    if mode == Mode::None {
        return failures;
    }
    check_encoder_owned(export, plain, &mut failures);
    check_capture(source, export, mode, &mut failures);
    check_parity(source, export, mode, &mut failures);
    check_rewritten(source, export, cs, &mut failures);
    failures
}

fn export(
    frame: &RawFrame,
    rgb: &[u8],
    format: &OutputFormat,
    cs: OutputColorSpace,
    mode: Mode,
) -> Vec<u8> {
    let location = match mode {
        Mode::All => Some(true),
        Mode::NoLocation => Some(false),
        Mode::None => None,
    };
    let meta = location.map(|location| ExportMetadata {
        exif: frame.exif.clone(),
        location,
    });
    let (embedded, _) = encode::embedded(meta.as_ref(), format, cs, (WIDTH, HEIGHT));
    encode::encode_from_rgb8(rgb, WIDTH, HEIGHT, format, cs, embedded.as_ref()).unwrap()
}

#[test]
fn cpu_render_embeds_the_requested_metadata() {
    let mut exif = little_exif::metadata::Metadata::new();
    exif.set_tag(ExifTag::Make("SONY".to_string()));
    exif.set_tag(ExifTag::GPSLatitude(vec![59u32.into(); 3]));
    let frame = synthetic_frame(WIDTH as usize, HEIGHT as usize);
    let cases = [
        (Some(exif.clone()), true, (true, true, 0)),
        (Some(exif), false, (true, false, 0)),
        (None, true, (false, false, 1)),
    ];
    for (source, location, expected) in cases {
        let opts = RenderOptions {
            max_edge: WIDTH,
            metadata: Some(ExportMetadata {
                exif: source,
                location,
            }),
            ..Default::default()
        };
        let out = cpu::render(&frame, &Edits::default(), &opts).unwrap();
        let tags = raw_pipeline::metadata::read(&out.bytes).unwrap_or_default();
        let make = (&tags)
            .into_iter()
            .any(|t| matches!(t, ExifTag::Make(v) if v == "SONY"));
        let gps = (&tags)
            .into_iter()
            .any(|t| matches!(t, ExifTag::GPSLatitude(_)));
        let got = (make, gps, out.metadata_warnings.len());
        if got != expected {
            panic!("location {location}: expected {expected:?}, got {got:?}");
        }
    }
}

#[test]
fn exports_carry_the_source_metadata() {
    let version = exiftool_version();
    if version < MIN_EXIFTOOL {
        panic!("exiftool {version} is too old; {MIN_EXIFTOOL} or newer reads JPEG XL boxes");
    }
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("export-metadata");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let sources: Vec<PathBuf> = fixtures().into_iter().chain(image_fixtures()).collect();
    if sources.is_empty() {
        panic!("no fixtures found");
    }
    let rgb: Vec<u8> = (0..WIDTH * HEIGHT * 3).map(|i| (i % 251) as u8).collect();
    let mut cases: Vec<Case> = Vec::new();
    for (source, path) in sources.iter().enumerate() {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let frame = match decode::decode(&std::fs::read(path).unwrap()) {
            Ok(frame) => frame,
            Err(e) => panic!("{name}: decode failed: {e}"),
        };
        let combos = FORMATS
            .iter()
            .enumerate()
            .flat_map(|f| MODES.map(|m| (f, m)));
        for ((format, (ext, output, cs)), mode) in combos {
            let out = dir.join(format!("{name}.{mode:?}.{ext}"));
            std::fs::write(&out, export(&frame, &rgb, output, *cs, mode)).unwrap();
            cases.push(Case {
                source,
                format,
                mode,
                path: out,
            });
        }
    }

    let all: Vec<&Path> = sources
        .iter()
        .map(PathBuf::as_path)
        .chain(cases.iter().map(|c| c.path.as_path()))
        .collect();
    let tags = read_tags(&dir, &all);
    let mut report: Vec<String> = Vec::new();
    for path in &sources {
        let warning = tags
            .get(path)
            .and_then(|t| find(t, |t| t.family0 == "ExifTool" && t.name == "Warning"))
            .and_then(|t| t.value.as_str());
        if let Some(warning) = warning
            && warning.contains("Install")
        {
            report.push(format!(
                "{}: exiftool cannot read it: {warning}",
                path.display()
            ));
        }
    }
    for case in &cases {
        let source_path = &sources[case.source];
        let plain = cases
            .iter()
            .find(|c| c.source == case.source && c.format == case.format && c.mode == Mode::None)
            .and_then(|c| tags.get(&c.path));
        let (Some(source), Some(export), Some(plain)) =
            (tags.get(source_path), tags.get(&case.path), plain)
        else {
            report.push(format!(
                "{}: exiftool returned nothing",
                case.path.display()
            ));
            continue;
        };
        let (ext, _, cs) = FORMATS[case.format];
        let name = source_path.file_name().unwrap().to_string_lossy();
        for (kind, detail) in check(source, export, plain, case.mode, cs) {
            report.push(format!("{name} -> {ext} {:?} [{kind}] {detail}", case.mode));
        }
    }
    std::fs::write(dir.join("report.txt"), report.join("\n")).unwrap();
    if !report.is_empty() {
        panic!(
            "{} metadata failures (exports in {}):\n{}",
            report.len(),
            dir.display(),
            report.join("\n")
        );
    }
}
