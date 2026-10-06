use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use raw_pipeline::{decode, encode, exif};
use raw_pipeline_testkit::fixtures::fixtures;

fn capture_date(meta: &Metadata) -> Option<String> {
    meta.into_iter().find_map(|t| match t {
        ExifTag::DateTimeOriginal(v) => Some(v.clone()),
        _ => None,
    })
}

#[test]
fn every_raw_fixture_exports_its_capture_exif() {
    let mut failures: Vec<String> = Vec::new();
    for p in fixtures() {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        let bytes = std::fs::read(&p).unwrap();
        let Some(meta) = decode::decode(&bytes).unwrap().exif else {
            failures.push(format!("{name}: no source exif"));
            continue;
        };
        let rgb = vec![128u8; 320 * 240 * 3];
        let mut jpeg = encode::encode_jpeg_rgb(
            encode::ImageRgb8 {
                rgb: &rgb,
                width: 320,
                height: 240,
            },
            85,
            raw_pipeline::frame::JpegSubsampling::Chroma420,
            raw_pipeline::frame::OutputColorSpace::SRgb,
        )
        .unwrap();
        exif::inject(&mut jpeg, &meta, little_exif::filetype::FileExtension::JPEG).unwrap();
        if &jpeg[..2] != b"\xff\xd8" || &jpeg[jpeg.len() - 2..] != b"\xff\xd9" {
            failures.push(format!("{name}: not a complete jpeg"));
        }
        if jpeg.len() > 1_000_000 {
            failures.push(format!(
                "{name}: injected jpeg bloated ({} bytes); embedded preview leaked",
                jpeg.len()
            ));
        }
        let written = exif::parse(&jpeg).as_ref().and_then(capture_date);
        if written.is_none() || written != capture_date(&meta) {
            failures.push(format!(
                "{name}: capture date {:?} written as {written:?}",
                capture_date(&meta)
            ));
        }
    }
    if !failures.is_empty() {
        panic!("{}", failures.join("\n"));
    }
}
