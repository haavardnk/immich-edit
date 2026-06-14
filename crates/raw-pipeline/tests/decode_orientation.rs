use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;
use raw_pipeline::frame::JpegSubsampling;
use raw_pipeline::{cpu, decode, edits::Edits, encode, frame::RenderOptions};

fn jpeg_with_orientation(width: usize, height: usize, orientation: u16) -> Vec<u8> {
    let rgb = vec![128u8; width * height * 3];
    let mut jpeg = encode::encode_jpeg_rgb(
        encode::ImageRgb8 {
            rgb: &rgb,
            width: width as u32,
            height: height as u32,
        },
        90,
        JpegSubsampling::Chroma420,
    )
    .unwrap();
    let mut meta = Metadata::new();
    meta.set_tag(ExifTag::Orientation(vec![orientation]));
    meta.write_to_vec(&mut jpeg, FileExtension::JPEG).unwrap();
    jpeg
}

fn rendered_dims(jpeg: &[u8]) -> (usize, usize) {
    let frame = decode::decode(jpeg).unwrap();
    let opts = RenderOptions {
        max_edge: 256,
        ..Default::default()
    };
    let out = cpu::render(&frame, &Edits::default(), &opts).unwrap();
    (out.width as usize, out.height as usize)
}

#[test]
fn jpeg_orientation_6_renders_portrait() {
    let jpeg = jpeg_with_orientation(40, 20, 6);
    let (w, h) = rendered_dims(&jpeg);
    if h <= w {
        panic!("orientation 6 should transpose to portrait, got {w}x{h}");
    }
}

#[test]
fn jpeg_orientation_1_stays_landscape() {
    let jpeg = jpeg_with_orientation(40, 20, 1);
    let (w, h) = rendered_dims(&jpeg);
    if w <= h {
        panic!("orientation 1 landscape should stay landscape, got {w}x{h}");
    }
}
