use raw_pipeline::{encode, exif};
use raw_pipeline_testkit::fixtures::fixtures;

#[test]
fn injected_jpeg_is_valid() {
    let mut tested = 0;
    for p in fixtures() {
        let bytes = std::fs::read(&p).unwrap();
        let Some(meta) = exif::parse(&bytes) else {
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
        if &jpeg[..2] != b"\xff\xd8" {
            panic!("{:?}: not jpeg", p.file_name());
        }
        if &jpeg[jpeg.len() - 2..] != b"\xff\xd9" {
            panic!("{:?}: missing EOI", p.file_name());
        }
        if jpeg.len() > 1_000_000 {
            panic!(
                "{:?}: injected jpeg bloated ({} bytes); embedded preview leaked",
                p.file_name(),
                jpeg.len()
            );
        }
        tested += 1;
    }
    if tested == 0 {
        eprintln!("no fixtures parsed exif");
    }
}
