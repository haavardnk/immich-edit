use super::Develop;
use super::bad_pixels;
use super::dng::DngCorrections;
use super::highlights::{reconstruct_mosaic, reconstruct_rgb};
use super::mosaic::{ActiveArea, Mosaic};
use crate::PipelineError;
use crate::frame::{FrameId, FrameMeta, RawFrame};
use rawler::cfa::CFA;
use rawler::imgop::chromatic_adaption::adapt_bradford;
use rawler::imgop::develop::{Intermediate, ProcessingStep, RawDevelop};
use rawler::imgop::matrix::transform_1d;
use rawler::imgop::sensor::SensorType;
use rawler::imgop::xyz::Illuminant;
use rawler::rawimage::{RawImageData, RawPhotometricInterpretation};
use rayon::prelude::*;

const MATRIX_ILLUMINANTS: [Illuminant; 9] = [
    Illuminant::D65,
    Illuminant::A,
    Illuminant::B,
    Illuminant::C,
    Illuminant::D50,
    Illuminant::D55,
    Illuminant::D75,
    Illuminant::Daylight,
    Illuminant::Flash,
];

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
        .and_then(crate::metadata::exif::orientation)
        .unwrap_or_else(|| raw_image.orientation.to_flips());
    (wb_coeffs, xyz_to_cam, color_matrices, orientation)
}

fn illuminant_to_cct(illu: &Illuminant) -> f32 {
    use Illuminant::*;
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
    let Some((illu, matrix)) = raw_image.color_matrix_find_first(MATRIX_ILLUMINANTS) else {
        return;
    };
    if matrix.len() % 3 != 0 {
        return;
    }
    let adapted = match (illu, transform_1d::<3, 3>(&matrix)) {
        (Illuminant::D65, _) | (_, None) => None,
        (illu, Some(rows)) => Some(
            adapt_bradford(&illu, &Illuminant::D65, &rows)
                .as_flattened()
                .to_vec(),
        ),
    };
    let matrix = adapted.unwrap_or(matrix);
    let components = (matrix.len() / 3).min(4);
    let mut xyz_to_cam = [[0.0f32; 3]; 4];
    for i in 0..components {
        for j in 0..3 {
            xyz_to_cam[i][j] = matrix[i * 3 + j];
        }
    }
    raw_image.xyz_to_cam = xyz_to_cam;
}

fn keeps_mosaic(raw_image: &rawler::RawImage, develop: Develop) -> bool {
    let RawPhotometricInterpretation::Cfa(config) = &raw_image.photometric else {
        return false;
    };
    let mosaic = match config.sensor {
        SensorType::Xtrans => true,
        SensorType::Bayer => develop == Develop::Fast && config.cfa.is_rgb(),
    };
    raw_image.cpp == 1 && mosaic
}

pub(super) fn frame_bytes(raw_image: &rawler::RawImage, develop: Develop) -> u64 {
    let channels = if keeps_mosaic(raw_image, develop) {
        1
    } else {
        3
    };
    let ActiveArea { size: [w, h], .. } = ActiveArea::of(raw_image);
    (w * h * channels * size_of::<f32>()) as u64
}

pub(super) fn decode_raw_fast(
    raw_image: rawler::RawImage,
    exif: Option<little_exif::metadata::Metadata>,
    dng: &DngCorrections,
) -> crate::PipelineResult<RawFrame> {
    if raw_image.cpp != 1 {
        return decode_raw_quality(raw_image, exif, dng);
    }
    match &raw_image.photometric {
        RawPhotometricInterpretation::Cfa(config)
            if config.sensor == SensorType::Bayer && config.cfa.is_rgb() =>
        {
            let cfa = config.cfa.clone();
            mosaic_frame(raw_image, exif, cfa, dng)
        }
        _ => decode_raw_quality(raw_image, exif, dng),
    }
}

fn active_pattern(raw_image: &rawler::RawImage, cfa: &CFA) -> String {
    let [x, y] = ActiveArea::of(raw_image).origin;
    cfa.shift(x, y).name
}

fn mosaic_frame(
    mut raw_image: rawler::RawImage,
    exif: Option<little_exif::metadata::Metadata>,
    cfa: CFA,
    dng: &DngCorrections,
) -> crate::PipelineResult<RawFrame> {
    let (wb_coeffs, xyz_to_cam, color_matrices, orientation) =
        extract_common(&mut raw_image, &exif);

    let flagged = dng.flagged_pixels(&raw_image);
    let (mut data, width, height) = scaled_mosaic(&raw_image);
    let cfa_pattern = active_pattern(&raw_image, &cfa);
    bad_pixels::repair(
        &mut Mosaic::whole(&mut data, width, height, &cfa_pattern),
        &flagged,
    );

    let capture_sigma = crate::capture_sigma::estimate(&data, width, height, &cfa_pattern);
    reconstruct_mosaic(&mut data, width, height, &cfa_pattern, wb_coeffs);
    dng.apply_gains(&mut Mosaic::whole(&mut data, width, height, &cfa_pattern));

    Ok(RawFrame {
        meta: FrameMeta {
            width,
            height,
            wb_coeffs,
            xyz_to_cam,
            color_matrices,
            orientation,
            is_raw: true,
            capture_sigma,
            model: raw_image.clean_model.clone(),
        },
        cfa_pattern,
        bps: 16,
        data,
        cpp: 1,
        exif,
        embedded_lens: dng.embedded_lens(true),
        id: FrameId::fresh(),
    })
}

pub(super) fn scaled_mosaic(raw_image: &rawler::RawImage) -> (Vec<f32>, usize, usize) {
    let ActiveArea {
        origin: [x0, y0],
        size: [width, height],
    } = ActiveArea::of(raw_image);
    let data = match &raw_image.data {
        RawImageData::Integer(src) => scale_rows(src, raw_image, (x0, y0), (width, height)),
        RawImageData::Float(src) => scale_rows(src, raw_image, (x0, y0), (width, height)),
    };
    (data, width, height)
}

fn scale_rows<T: Copy + Into<f32> + Sync>(
    src: &[T],
    raw_image: &rawler::RawImage,
    (x0, y0): (usize, usize),
    (width, height): (usize, usize),
) -> Vec<f32> {
    let black = raw_image.blacklevel.as_bayer_array();
    let white = raw_image.whitelevel.as_bayer_array();
    let range: [f32; 4] = std::array::from_fn(|i| white[i] - black[i]);
    let stride = raw_image.width;
    let mut out = vec![0.0f32; width * height];
    out.par_chunks_exact_mut(width.max(1))
        .enumerate()
        .for_each(|(y, row)| {
            let sy = y0 + y;
            let start = sy * stride + x0;
            let parity = (sy & 1) * 2;
            for (x, (dst, &v)) in row.iter_mut().zip(&src[start..start + width]).enumerate() {
                let i = parity + ((x0 + x) & 1);
                let shifted = v.into() - black[i];
                let clipped = if shifted.is_sign_negative() {
                    0.0
                } else {
                    shifted
                };
                *dst = clipped / range[i];
            }
        });
    out
}

pub(super) fn decode_raw_quality(
    mut raw_image: rawler::RawImage,
    exif: Option<little_exif::metadata::Metadata>,
    dng: &DngCorrections,
) -> crate::PipelineResult<RawFrame> {
    if raw_image.cpp == 1
        && let RawPhotometricInterpretation::Cfa(config) = &raw_image.photometric
    {
        match config.sensor {
            SensorType::Xtrans => {
                let cfa = config.cfa.clone();
                return decode_raw_xtrans(raw_image, exif, cfa, dng);
            }
            SensorType::Bayer if !config.cfa.is_rgb() && config.cfa.unique_colors() != 4 => {
                return Err(PipelineError::Unsupported(format!(
                    "unsupported CFA pattern '{}'",
                    config.cfa.name
                )));
            }
            SensorType::Bayer => {}
        }
    }

    if raw_image.active_area.is_some() && raw_image.fuji_rotation_width.is_some() {
        return Err(PipelineError::Unsupported(format!(
            "rotated Fuji sensor not supported: '{}'",
            raw_image.clean_model
        )));
    }

    let (wb_coeffs, xyz_to_cam, color_matrices, orientation) =
        extract_common(&mut raw_image, &exif);

    let mosaic_cfa = match &raw_image.photometric {
        RawPhotometricInterpretation::Cfa(config) if raw_image.cpp == 1 && config.cfa.is_rgb() => {
            Some(config.cfa.clone())
        }
        _ => None,
    };
    let mut steps = vec![ProcessingStep::Demosaic, ProcessingStep::CropActiveArea];
    let gains_applied = match &mosaic_cfa {
        Some(cfa) => correct_full_mosaic(&mut raw_image, cfa, wb_coeffs, dng)?,
        None => {
            steps.insert(0, ProcessingStep::Rescale);
            raw_image.cpp == 3
        }
    };
    if !gains_applied && dng.has_gains() {
        tracing::debug!(model = %raw_image.clean_model, "skipping DNG gains on this pixel layout");
    }
    let intermediate = RawDevelop { steps }
        .develop_intermediate(&raw_image)
        .map_err(|e| PipelineError::Decode(format!("develop: {e}")))?;

    let (mut data, width, height) = match intermediate {
        Intermediate::ThreeColor(pixels) => {
            let w = pixels.width;
            let h = pixels.height;
            (pixels.into_inner().into_flattened(), w, h)
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
    if raw_image.cpp == 3 {
        reconstruct_rgb(&mut data, width, height, wb_coeffs);
        dng.apply_gains_rgb(&mut data, width, height);
    }

    Ok(RawFrame {
        meta: FrameMeta {
            width,
            height,
            wb_coeffs,
            xyz_to_cam,
            color_matrices,
            orientation,
            is_raw: true,
            capture_sigma: None,
            model: raw_image.clean_model.clone(),
        },
        cfa_pattern: String::new(),
        bps: 16,
        data,
        cpp: 3,
        exif,
        embedded_lens: dng.embedded_lens(gains_applied),
        id: FrameId::fresh(),
    })
}

fn correct_full_mosaic(
    raw_image: &mut rawler::RawImage,
    cfa: &CFA,
    wb_coeffs: [f32; 4],
    dng: &DngCorrections,
) -> crate::PipelineResult<bool> {
    let flagged = dng.flagged_pixels(raw_image);
    raw_image
        .apply_scaling()
        .map_err(|e| PipelineError::Decode(format!("scale: {e}")))?;
    let area = ActiveArea::of(raw_image);
    let active = active_pattern(raw_image, cfa);
    let stride = raw_image.width;
    let height = raw_image.height;
    let RawImageData::Float(data) = &mut raw_image.data else {
        return Ok(false);
    };
    if !area.fits(stride, height) {
        tracing::warn!(?area, stride, height, "active area exceeds the raw image");
        reconstruct_mosaic(data, stride, height, &cfa.name, wb_coeffs);
        return Ok(false);
    }
    bad_pixels::repair(&mut Mosaic::window(data, stride, area, &active), &flagged);
    reconstruct_mosaic(data, stride, height, &cfa.name, wb_coeffs);
    dng.apply_gains(&mut Mosaic::window(data, stride, area, &active));
    Ok(true)
}

fn decode_raw_xtrans(
    raw_image: rawler::RawImage,
    exif: Option<little_exif::metadata::Metadata>,
    cfa: CFA,
    dng: &DngCorrections,
) -> crate::PipelineResult<RawFrame> {
    if crate::cpu::demosaic::parse_xtrans(&cfa.name).is_none() {
        return Err(PipelineError::Unsupported(format!(
            "unsupported 6x6 CFA pattern '{}'",
            cfa.name
        )));
    }
    mosaic_frame(raw_image, exif, cfa, dng)
}
