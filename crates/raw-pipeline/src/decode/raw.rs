use crate::PipelineError;
use crate::frame::RawFrame;
use rawler::imgop::develop::{Intermediate, ProcessingStep, RawDevelop};
use rawler::rawimage::RawPhotometricInterpretation;

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

pub(super) fn decode_raw_fast(
    mut raw_image: rawler::RawImage,
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    if raw_image.cpp != 1 {
        return decode_raw_quality(raw_image, exif);
    }
    let cfa_name = match &raw_image.photometric {
        RawPhotometricInterpretation::Cfa(config)
            if config.cfa.is_rgb() && config.cfa.width == 2 && config.cfa.height == 2 =>
        {
            config.cfa.name.clone()
        }
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

    let capture_sigma = crate::capture_sigma::estimate(&data, width, height, &cfa_pattern);

    Ok(RawFrame {
        width,
        height,
        cfa_pattern,
        bps: 16,
        wb_coeffs,
        xyz_to_cam,
        color_matrices,
        data,
        cpp: 1,
        orientation,
        is_raw: true,
        capture_sigma,
        model: raw_image.clean_model.clone(),
        exif,
    })
}

pub(super) fn decode_raw_quality(
    mut raw_image: rawler::RawImage,
    exif: Option<little_exif::metadata::Metadata>,
) -> crate::PipelineResult<RawFrame> {
    if raw_image.cpp == 1
        && let RawPhotometricInterpretation::Cfa(config) = &raw_image.photometric
    {
        match (config.cfa.width, config.cfa.height) {
            (0, 0) | (2, 2) => {}
            (6, 6) => {
                let cfa_name = config.cfa.name.clone();
                return decode_raw_xtrans(raw_image, exif, &cfa_name);
            }
            (w, h) => {
                return Err(PipelineError::Unsupported(format!(
                    "unsupported {w}x{h} CFA pattern '{}'",
                    config.cfa.name
                )));
            }
        }
    }

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
        data,
        cpp: 3,
        orientation,
        is_raw: true,
        capture_sigma: None,
        model: raw_image.clean_model.clone(),
        exif,
    })
}

fn decode_raw_xtrans(
    mut raw_image: rawler::RawImage,
    exif: Option<little_exif::metadata::Metadata>,
    cfa_name: &str,
) -> crate::PipelineResult<RawFrame> {
    let Some(pattern) = crate::cpu::demosaic::parse_xtrans(cfa_name) else {
        return Err(PipelineError::Unsupported(format!(
            "unsupported 6x6 CFA pattern '{cfa_name}'"
        )));
    };

    let (wb_coeffs, xyz_to_cam, color_matrices, orientation) =
        extract_common(&mut raw_image, &exif);

    let develop = RawDevelop {
        steps: vec![ProcessingStep::Rescale],
    };
    let intermediate = develop
        .develop_intermediate(&raw_image)
        .map_err(|e| PipelineError::Decode(format!("develop: {e}")))?;

    let Intermediate::Monochrome(pixels) = intermediate else {
        return Err(PipelineError::Decode(
            "X-Trans develop did not yield a mosaic".into(),
        ));
    };

    let (data, width, height, pattern) = if let Some(area) = raw_image.active_area {
        let shifted = crate::cpu::demosaic::shift_xtrans(&pattern, area.p.x, area.p.y);
        let cropped = pixels.crop(area);
        let w = cropped.width;
        let h = cropped.height;
        (cropped.into_inner(), w, h, shifted)
    } else {
        let w = pixels.width;
        let h = pixels.height;
        (pixels.into_inner(), w, h, pattern)
    };

    let cfa_pattern: String = pattern.iter().map(|b| *b as char).collect();
    let capture_sigma = crate::capture_sigma::estimate(&data, width, height, &cfa_pattern);

    Ok(RawFrame {
        width,
        height,
        cfa_pattern,
        bps: 16,
        wb_coeffs,
        xyz_to_cam,
        color_matrices,
        data,
        cpp: 1,
        orientation,
        is_raw: true,
        capture_sigma,
        model: raw_image.clean_model.clone(),
        exif,
    })
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
