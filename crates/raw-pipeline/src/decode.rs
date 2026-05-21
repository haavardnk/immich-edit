use crate::frame::RawFrame;
use crate::PipelineError;

pub fn decode(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    let source = rawler::rawsource::RawSource::new_from_slice(data);
    let params = rawler::decoders::RawDecodeParams::default();
    let mut raw_image = rawler::decode(&source, &params)
        .map_err(|e| PipelineError::Decode(format!("{e}")))?;

    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        raw_image.apply_scaling()
    }));

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
    })
}
