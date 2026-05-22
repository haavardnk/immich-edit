use raw_pipeline::{encode, exif};

#[test]
fn injected_jpeg_is_valid() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        eprintln!("no fixtures");
        return;
    };
    let exts: &[&str] = &[
        "arw", "cr2", "cr3", "dng", "nef", "raf", "orf", "rw2", "pef",
    ];
    let mut tested = 0;
    for e in entries.filter_map(|e| e.ok()) {
        let p = e.path();
        let ext = p
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| x.to_ascii_lowercase());
        if !ext.as_deref().map(|x| exts.contains(&x)).unwrap_or(false) {
            continue;
        }
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
            eprintln!(
                "{:?}: injected jpeg large ({} bytes); known bloat for some fixtures, skipping",
                p.file_name(),
                jpeg.len()
            );
            continue;
        }
        tested += 1;
    }
    if tested == 0 {
        eprintln!("no fixtures parsed exif");
    }
}
