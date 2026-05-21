use crate::edits::Edits;
use crate::encode::encode_jpeg;
use crate::frame::{RawFrame, RenderOptions, RenderedImage};
use crate::histogram::Histogram;
use super::demosaic;
use super::transform;

pub fn render(
    frame: &RawFrame,
    edits: &Edits,
    options: &RenderOptions,
) -> crate::PipelineResult<RenderedImage> {
    let edits = edits.clamped();

    let mut rgb = demosaic::demosaic(frame);
    let mut w = frame.width;
    let mut h = frame.height;

    apply_wb(&mut rgb, frame, &edits);
    apply_exposure(&mut rgb, &edits);
    apply_highlights_shadows(&mut rgb, &edits);
    apply_contrast(&mut rgb, &edits);
    apply_saturation(&mut rgb, &edits);

    let mut steps = edits.rotate / 90;
    while steps > 0 {
        let (rotated, nw, nh) = transform::rotate_90(&rgb, w, h);
        rgb = rotated;
        w = nw;
        h = nh;
        steps -= 1;
    }

    if edits.flip_h {
        transform::flip_horizontal(&mut rgb, w, h);
    }
    if edits.flip_v {
        transform::flip_vertical(&mut rgb, w, h);
    }

    if let Some(crop) = &edits.crop {
        let (cropped, cw, ch) = transform::crop(&rgb, w, h, crop.x, crop.y, crop.width, crop.height);
        rgb = cropped;
        w = cw;
        h = ch;
    }

    let (rgb, w, h) = transform::resize(&rgb, w, h, options.max_edge);

    let histogram = Histogram::from_rgb(&rgb, w, h);

    let mut srgb = rgb;
    linear_to_srgb(&mut srgb);

    let rgb_u8: Vec<u8> = srgb.iter().map(|&v| (v.clamp(0.0, 1.0) * 255.0) as u8).collect();

    let jpeg = encode_jpeg(&rgb_u8, w as u32, h as u32, 85)?;

    Ok(RenderedImage {
        jpeg,
        histogram,
        width: w as u32,
        height: h as u32,
        renderer: "cpu".into(),
    })
}

fn apply_wb(rgb: &mut [f32], frame: &RawFrame, edits: &Edits) {
    let mut coeffs = frame.wb_coeffs;
    if coeffs[0] == 0.0 && coeffs[1] == 0.0 && coeffs[2] == 0.0 {
        coeffs = [1.0, 1.0, 1.0, 1.0];
    }

    let g_coeff = coeffs[1];
    if g_coeff > 0.0 {
        coeffs[0] /= g_coeff;
        coeffs[1] = 1.0;
        coeffs[2] /= g_coeff;
        coeffs[3] /= g_coeff;
    }

    let temp_shift = edits.wb_temp as f32 / 100.0;
    let tint_shift = edits.wb_tint as f32 / 100.0;

    coeffs[0] *= 1.0 + temp_shift * 0.5;
    coeffs[2] *= 1.0 - temp_shift * 0.5;
    coeffs[1] *= 1.0 - tint_shift * 0.3;

    let pixel_count = rgb.len() / 3;
    for i in 0..pixel_count {
        let idx = i * 3;
        rgb[idx] *= coeffs[0];
        rgb[idx + 1] *= coeffs[1];
        rgb[idx + 2] *= coeffs[2];
    }
}

fn apply_exposure(rgb: &mut [f32], edits: &Edits) {
    if edits.exposure_ev == 0.0 {
        return;
    }
    let factor = 2.0f32.powf(edits.exposure_ev as f32);
    for v in rgb.iter_mut() {
        *v *= factor;
    }
}

fn apply_highlights_shadows(rgb: &mut [f32], edits: &Edits) {
    if edits.highlights == 0.0 && edits.shadows == 0.0 {
        return;
    }
    let hl = edits.highlights as f32 / 100.0;
    let sh = edits.shadows as f32 / 100.0;

    for v in rgb.iter_mut() {
        let x = v.clamp(0.0, 2.0);
        if x > 0.5 {
            *v = x + hl * (1.0 - x) * 0.5;
        } else {
            *v = x + sh * x * 0.5;
        }
    }
}

fn apply_contrast(rgb: &mut [f32], edits: &Edits) {
    if edits.contrast == 0.0 {
        return;
    }
    let factor = 1.0 + edits.contrast as f32 / 100.0;
    for v in rgb.iter_mut() {
        *v = (*v - 0.5) * factor + 0.5;
    }
}

fn apply_saturation(rgb: &mut [f32], edits: &Edits) {
    if edits.saturation == 0.0 {
        return;
    }
    let factor = 1.0 + edits.saturation as f32 / 100.0;
    let pixel_count = rgb.len() / 3;
    for i in 0..pixel_count {
        let idx = i * 3;
        let luma = 0.2126 * rgb[idx] + 0.7152 * rgb[idx + 1] + 0.0722 * rgb[idx + 2];
        rgb[idx] = luma + (rgb[idx] - luma) * factor;
        rgb[idx + 1] = luma + (rgb[idx + 1] - luma) * factor;
        rgb[idx + 2] = luma + (rgb[idx + 2] - luma) * factor;
    }
}

fn linear_to_srgb(rgb: &mut [f32]) {
    for v in rgb.iter_mut() {
        *v = if *v <= 0.0031308 {
            *v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
    }
}
