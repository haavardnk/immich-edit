use crate::PipelineError;
use crate::frame::RawFrame;
use rawler::imgop::develop::{Intermediate, ProcessingStep, RawDevelop};
use rawler::rawimage::RawPhotometricInterpretation;

pub fn decode(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    let exif = crate::exif::parse(data);
    let source = rawler::rawsource::RawSource::new_from_slice(data);
    let params = rawler::decoders::RawDecodeParams::default();
    match rawler::decode(&source, &params) {
        Ok(raw_image) => decode_raw_fast(raw_image, exif),
        Err(e) => {
            let msg = format!("{e}");
            if msg.contains("No decoder found") {
                decode_image(data, exif)
            } else {
                Err(PipelineError::Decode(msg))
            }
        }
    }
}

pub fn decode_quality(data: &[u8]) -> crate::PipelineResult<RawFrame> {
    let exif = crate::exif::parse(data);
    let source = rawler::rawsource::RawSource::new_from_slice(data);
    let params = rawler::decoders::RawDecodeParams::default();
    match rawler::decode(&source, &params) {
        Ok(raw_image) => decode_raw_quality(raw_image, exif),
        Err(e) => {
            let msg = format!("{e}");
            if msg.contains("No decoder found") {
                decode_image(data, exif)
            } else {
                Err(PipelineError::Decode(msg))
            }
        }
    }
}

type ExtractedMeta = (
    [f32; 4],
    [[f32; 3]; 4],
    Vec<(f32, [[f32; 3]; 4])>,
    crate::frame::OrientFlips,
);

fn extract_common(
    raw_image: &mut rawler::RawImage,
    exif: &Option<little_exif::metadata::Metadata>,
) -> ExtractedMeta {
    let wb_coeffs = raw_image.wb_coeffs;
    let color_matrices = extract_color_matrices(raw_image);
    populate_xyz_to_cam_from_color_matrix(raw_image);
    let xyz_to_cam = raw_image.xyz_to_cam;
    let orientation = exif
        .as_ref()
        .and_then(crate::exif::orientation)
        .unwrap_or_else(|| raw_image.orientation.to_flips());
    (wb_coeffs, xyz_to_cam, color_matrices, orientation)
}

fn illuminant_to_cct(illu: &rawler::imgop::xyz::Illuminant) -> f32 {
    use rawler::imgop::xyz::Illuminant::*;
    match illu {
        A | Tungsten => 2856.0,
        B => 4874.0,
        C => 6774.0,
        D50 => 5003.0,
        D55 => 5503.0,
        D65 => 6504.0,
        D75 => 7504.0,
        Daylight | FineWeather | Flash => 5500.0,
        Fluorescent => 4150.0,
        CloudyWeather => 6500.0,
        Shade => 7500.0,
        DaylightFluorescent => 6430.0,
        DaylightWhiteFluorescent => 5000.0,
        CoolWhiteFluorescent => 4150.0,
        WhiteFluorescent => 3450.0,
        IsoStudioTungsten => 3200.0,
        Unknown => 6504.0,
    }
}

fn extract_color_matrices(raw_image: &rawler::RawImage) -> Vec<(f32, [[f32; 3]; 4])> {
    let mut result = Vec::new();
    for (illu, matrix) in &raw_image.color_matrix {
        if matrix.len() % 3 != 0 {
            continue;
        }
        let components = (matrix.len() / 3).min(4);
        let mut xyz_to_cam = [[0.0f32; 3]; 4];
        for i in 0..components {
            for j in 0..3 {
                xyz_to_cam[i][j] = matrix[i * 3 + j];
            }
        }
        let cct = illuminant_to_cct(illu);
        result.push((cct, xyz_to_cam));
    }
    result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    result
}

fn populate_xyz_to_cam_from_color_matrix(raw_image: &mut rawler::RawImage) {
    if raw_image
        .xyz_to_cam
        .iter()
        .any(|row| row.iter().any(|v| *v != 0.0))
    {
        return;
    }
    let matrix = raw_image
        .color_matrix
        .iter()
        .find(|(illu, _)| **illu == rawler::imgop::xyz::Illuminant::D65)
        .map(|(_, m)| m)
        .or_else(|| raw_image.color_matrix.values().next());
    let Some(matrix) = matrix else {
        return;
    };
    if matrix.len() % 3 != 0 {
        return;
    }
    let components = (matrix.len() / 3).min(4);
    let mut xyz_to_cam = [[0.0f32; 3]; 4];
    for i in 0..components {
        for j in 0..3 {
            xyz_to_cam[i][j] = matrix[i * 3 + j];
        }
    }
    raw_image.xyz_to_cam = xyz_to_cam;
}

fn decode_raw_fast(
    mut raw_image: rawler::RawImage,
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    if raw_image.cpp != 1 {
        return decode_raw_quality(raw_image, exif);
    }
    let cfa_name = match &raw_image.photometric {
        RawPhotometricInterpretation::Cfa(config) if config.cfa.is_rgb() => config.cfa.name.clone(),
        _ => return decode_raw_quality(raw_image, exif),
    };

    let (wb_coeffs, xyz_to_cam, color_matrices, orientation) =
        extract_common(&mut raw_image, &exif);

    let develop = RawDevelop {
        steps: vec![ProcessingStep::Rescale],
    };
    let intermediate = develop
        .develop_intermediate(&raw_image)
        .map_err(|e| PipelineError::Decode(format!("develop: {e}")))?;

    let pixels = match intermediate {
        Intermediate::Monochrome(p) => p,
        _ => return decode_raw_quality(raw_image, exif),
    };

    let (data, width, height, cfa_pattern) = if let Some(area) = raw_image.active_area {
        let cropped = pixels.crop(area);
        let shifted = shift_cfa(&cfa_name, area.p.x, area.p.y);
        let w = cropped.width;
        let h = cropped.height;
        (cropped.into_inner(), w, h, shifted)
    } else {
        let w = pixels.width;
        let h = pixels.height;
        (pixels.into_inner(), w, h, cfa_name)
    };

    Ok(RawFrame {
        width,
        height,
        cfa_pattern,
        bps: 16,
        wb_coeffs,
        xyz_to_cam,
        color_matrices,
        black_levels: [0.0; 4],
        white_levels: [1.0; 4],
        data,
        cpp: 1,
        orientation,
        is_raw: true,
        exif,
    })
}

fn decode_raw_quality(
    mut raw_image: rawler::RawImage,
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    let (wb_coeffs, xyz_to_cam, color_matrices, orientation) =
        extract_common(&mut raw_image, &exif);

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
        xyz_to_cam,
        color_matrices,
        black_levels: [0.0; 4],
        white_levels: [1.0; 4],
        data,
        cpp: 3,
        orientation,
        is_raw: true,
        exif,
    })
}

fn decode_image(
    data: &[u8],
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
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

    let linear: Vec<f32> = rgb_buf
        .iter()
        .map(|&v| srgb_to_linear(v as f32 / 255.0))
        .collect();

    Ok(RawFrame {
        width,
        height,
        cfa_pattern: String::new(),
        bps: 8,
        wb_coeffs: [1.0, 1.0, 1.0, 1.0],
        xyz_to_cam: [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
        ],
        color_matrices: Vec::new(),
        black_levels: [0.0; 4],
        white_levels: [1.0; 4],
        data: linear,
        cpp: 3,
        orientation: (false, false, false),
        is_raw: false,
        exif,
    })
}

fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn shift_cfa(cfa: &str, dx: usize, dy: usize) -> String {
    let b = cfa.as_bytes();
    if b.len() < 4 {
        return cfa.to_string();
    }
    let get = |x: usize, y: usize| -> u8 { b[((y % 2) * 2 + (x % 2)) % 4] };
    String::from_utf8(vec![
        get(dx, dy),
        get(dx + 1, dy),
        get(dx, dy + 1),
        get(dx + 1, dy + 1),
    ])
    .unwrap_or_else(|_| cfa.to_string())
}
