use crate::PipelineError;
use crate::frame::RawFrame;

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

fn decode_raw(mut raw_image: rawler::RawImage) -> crate::PipelineResult<RawFrame> {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| raw_image.apply_scaling()));

    let width = raw_image.width;
    let height = raw_image.height;
    let cpp = raw_image.cpp;
    let bps = raw_image.bps;
    let wb_coeffs = raw_image.wb_coeffs;
    let cam_to_xyz = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        raw_image.cam_to_xyz_normalized()
    }))
    .unwrap_or([[0.0; 4]; 3]);
    let cfa_pattern = raw_image.camera.cfa.name.clone();
    let orientation = raw_image.orientation.to_flips();

    let black_levels = [0.0f32; 4];
    let white_levels = [1.0f32; 4];

    let data = match &raw_image.data {
        rawler::RawImageData::Float(d) => d.clone(),
        rawler::RawImageData::Integer(d) => {
            let max = ((1u32 << bps.min(16)) - 1) as f32;
            d.iter().map(|&v| v as f32 / max).collect()
        }
    };

    Ok(RawFrame {
        width,
        height,
        cfa_pattern,
        bps,
        wb_coeffs,
        cam_to_xyz,
        black_levels,
        white_levels,
        data,
        cpp,
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
