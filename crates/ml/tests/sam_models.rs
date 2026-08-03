use std::path::{Path, PathBuf};

use ml::runtime::SessionConfig;
use ml::{BoxPrompt, ClickPoint, RuntimeMode, SamDecoder, SamEncoder};

fn fixture(name: &str) -> Option<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    path.exists().then_some(path)
}

fn disc(width: u32, height: u32) -> Vec<u8> {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let r = width.min(height) as f32 / 4.0;
    let mut rgb = vec![20u8; (width * height * 3) as usize];
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy > r * r {
                continue;
            }
            let i = ((y * width + x) * 3) as usize;
            rgb[i] = 240;
            rgb[i + 1] = 40;
            rgb[i + 2] = 40;
        }
    }
    rgb
}

fn round_trip(encoder_name: &str, decoder_name: &str, tensors: usize) {
    let (Some(encoder_path), Some(decoder_path)) = (fixture(encoder_name), fixture(decoder_name))
    else {
        eprintln!("skipping {encoder_name}: fixture missing");
        return;
    };
    let width = 640u32;
    let height = 480u32;
    let rgb = disc(width, height);
    let config = SessionConfig::default();

    let mut encoder = SamEncoder::open(&encoder_path, RuntimeMode::Cpu, &config).unwrap();
    let embedding = encoder.encode(&rgb, width, height).unwrap();
    assert_eq!(embedding.tensors.len(), tensors);
    assert_eq!((embedding.width, embedding.height), (width, height));
    assert!(embedding.tensors.iter().all(|t| !t.values.is_empty()));

    let mut decoder = SamDecoder::open(&decoder_path, RuntimeMode::Cpu, &config).unwrap();
    let mask = decoder
        .decode(
            &embedding,
            &[ClickPoint {
                x: width as f32 / 2.0,
                y: height as f32 / 2.0,
                positive: true,
            }],
        )
        .unwrap();

    assert_eq!((mask.width, mask.height), (width as usize, height as usize));
    let centre = mask.values[(height as usize / 2) * width as usize + width as usize / 2];
    let corner = mask.values[0];
    assert!(centre > 0.5, "{encoder_name} centre {centre}");
    assert!(corner < 0.5, "{encoder_name} corner {corner}");

    let r = width.min(height) as f32 / 4.0;
    let boxed = decoder
        .decode_with(
            &embedding,
            &[],
            Some(BoxPrompt {
                x0: width as f32 / 2.0 - r,
                y0: height as f32 / 2.0 - r,
                x1: width as f32 / 2.0 + r,
                y1: height as f32 / 2.0 + r,
            }),
        )
        .unwrap();
    assert_eq!(
        (boxed.width, boxed.height),
        (width as usize, height as usize)
    );
    let centre = boxed.values[(height as usize / 2) * width as usize + width as usize / 2];
    let corner = boxed.values[0];
    assert!(centre > 0.5, "{encoder_name} box centre {centre}");
    assert!(corner < 0.5, "{encoder_name} box corner {corner}");
}

#[test]
fn sam2_selects_the_clicked_region() {
    round_trip("sam2_tiny_encoder.onnx", "sam2_tiny_decoder.onnx", 3);
}

#[test]
fn mobilesam_selects_the_clicked_region() {
    round_trip("mobilesam_encoder.onnx", "mobilesam_decoder.onnx", 1);
}
