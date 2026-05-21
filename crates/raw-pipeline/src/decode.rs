use crate::PipelineError;
use crate::frame::RawFrame;
use rawler::imgop::develop::{Intermediate, ProcessingStep, RawDevelop};

pub fn decode(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    let source = rawler::rawsource::RawSource::new_from_slice(data);
    let params = rawler::decoders::RawDecodeParams::default();
    match rawler::decode(&source, &params) {
        Ok(raw_image) => decode_raw(raw_image),
        Err(e) => {
            let msg = format!("{e}");
            if msg.contains("No decoder found") {
                decode_image(data)
            } else {
                Err(PipelineError::Decode(msg))
            }
        }
    }
}

fn decode_raw(raw_image: rawler::RawImage) -> crate::PipelineResult<RawFrame> {
    let wb_coeffs = raw_image.wb_coeffs;
    let cam_to_xyz = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        raw_image.cam_to_xyz_normalized()
    }))
    .unwrap_or([[0.0; 4]; 3]);
    let orientation = raw_image.orientation.to_flips();

    let develop = RawDevelop {
        steps: vec![
            ProcessingStep::Rescale,
            ProcessingStep::Demosaic,
            ProcessingStep::CropActiveArea,
        ],
    };
    let intermediate = develop
        .develop_intermediate(&raw_image)
        .map_err(|e| PipelineError::Decode(format!("develop: {e}")))?;

    let (data, width, height) = match intermediate {
        Intermediate::ThreeColor(pixels) => {
            let w = pixels.width;
            let h = pixels.height;
            let flat: Vec<f32> = pixels.into_inner().into_iter().flatten().collect();
            (flat, w, h)
        }
        Intermediate::FourColor(pixels) => {
            let w = pixels.width;
            let h = pixels.height;
            let flat: Vec<f32> = pixels
                .into_inner()
                .into_iter()
                .flat_map(|p| [p[0], p[1], p[2]])
                .collect();
            (flat, w, h)
        }
        Intermediate::Monochrome(pixels) => {
            let w = pixels.width;
            let h = pixels.height;
            let flat: Vec<f32> = pixels
                .into_inner()
                .into_iter()
                .flat_map(|v| [v, v, v])
                .collect();
            (flat, w, h)
        }
    };

    Ok(RawFrame {
        width,
        height,
        cfa_pattern: String::new(),
        bps: 16,
        wb_coeffs,
        cam_to_xyz,
        black_levels: [0.0; 4],
        white_levels: [1.0; 4],
        data,
        cpp: 3,
        orientation,
    })
}

fn decode_image(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    let mut decompressor = turbojpeg::Decompressor::new()
        .map_err(|e| PipelineError::Decode(format!("jpeg init: {e}")))?;
    let header = decompressor
        .read_header(data)
        .map_err(|e| PipelineError::Decode(format!("jpeg header: {e}")))?;
    let width = header.width;
    let height = header.height;

    let mut rgb_buf = vec![0u8; width * height * 3];
    let image = turbojpeg::Image {
        pixels: rgb_buf.as_mut_slice(),
        width,
        pitch: width * 3,
        height,
        format: turbojpeg::PixelFormat::RGB,
    };
    decompressor
        .decompress(data, image)
        .map_err(|e| PipelineError::Decode(format!("jpeg decompress: {e}")))?;

    let linear: Vec<f32> = rgb_buf.iter().map(|&v| srgb_to_linear(v as f32 / 255.0)).collect();

    Ok(RawFrame {
        width,
        height,
        cfa_pattern: String::new(),
        bps: 8,
        wb_coeffs: [1.0, 1.0, 1.0, 1.0],
        cam_to_xyz: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]],
        black_levels: [0.0; 4],
        white_levels: [1.0; 4],
        data: linear,
        cpp: 3,
        orientation: (false, false, false),
    })
}

fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
