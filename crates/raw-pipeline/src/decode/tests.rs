use super::bitmap::{InputFormat, sniff_format};

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
