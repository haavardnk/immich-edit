use crate::cancel::{self, CancelToken};
use crate::cpu::{demosaic, transform};
use crate::edits::Edits;
use crate::encode::encode_jpeg;
use crate::frame::{RawFrame, RenderOptions, RenderedImage};
use crate::histogram::Histogram;
use crate::ops::LinearImage;
use crate::ops::{OpContext, default_registry};
use rayon::prelude::*;

pub fn render(
    frame: &RawFrame,
    edits: &Edits,
    options: &RenderOptions,
) -> crate::PipelineResult<RenderedImage> {
    render_with_cancel(frame, edits, options, None)
}

pub fn render_with_cancel(
    frame: &RawFrame,
    edits: &Edits,
    options: &RenderOptions,
    cancel: Option<&CancelToken>,
) -> crate::PipelineResult<RenderedImage> {
    let edits = edits.clamped();

    let (rgb, src_w, src_h) = if frame.cpp == 1 && !frame.cfa_pattern.is_empty() {
        let d = demosaic::malvar_he_cutler(
            &frame.data,
            frame.width,
            frame.height,
            &frame.cfa_pattern,
        );
        (d, frame.width, frame.height)
    } else {
        (frame.data.clone(), frame.width, frame.height)
    };

    let (rgb, w, h) = transform::apply_orientation(rgb, src_w, src_h, frame.orientation);

    let mut image = LinearImage::new(rgb, w, h);
    let cam_to_srgb = if frame.is_raw && !crate::color::is_unusable_matrix(&frame.xyz_to_cam) {
        crate::color::cam_to_srgb_matrix(frame.xyz_to_cam)
    } else {
        crate::color::identity_3x3()
    };
    let ctx = OpContext {
        wb_coeffs: frame.wb_coeffs,
        cam_to_srgb,
        is_raw: frame.is_raw,
    };

    let registry = default_registry();
    for op in registry.active(&edits) {
        cancel::check(cancel)?;
        op.apply_cpu(&mut image, &ctx, &edits)?;
    }

    cancel::check(cancel)?;
    let (rgb, w, h) = transform::resize(&image.rgb, image.width, image.height, options.max_edge);

    let mut srgb = rgb;
    cancel::check(cancel)?;
    apply_default_tone(&mut srgb);

    let rgb_u8: Vec<u8> = srgb
        .par_iter()
        .map(|&v| (v.clamp(0.0, 1.0) * 255.0) as u8)
        .collect();
    cancel::check(cancel)?;

    let (histogram, jpeg) = rayon::join(
        || Histogram::from_rgb_u8(&rgb_u8, w, h),
        || encode_jpeg(&rgb_u8, w as u32, h as u32, 85),
    );
    let jpeg = jpeg?;

    Ok(RenderedImage {
        jpeg,
        histogram,
        width: w as u32,
        height: h as u32,
        renderer: "cpu".into(),
    })
}

const S_CURVE_BLEND: f32 = 0.15;

pub(crate) fn default_tone(v: f32) -> f32 {
    let lin = v.clamp(0.0, 1.0);
    let srgb = srgb_oetf(lin);
    let s = srgb * srgb * (3.0 - 2.0 * srgb);
    let out = srgb + (s - srgb) * S_CURVE_BLEND;
    out.clamp(0.0, 1.0)
}

fn srgb_oetf(v: f32) -> f32 {
    if v <= 0.003_130_8 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

fn apply_default_tone(rgb: &mut [f32]) {
    rgb.par_iter_mut().for_each(|v| *v = default_tone(*v));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tone_preserves_endpoints() {
        if default_tone(0.0).abs() > 1e-4 {
            panic!("expected 0 at 0");
        }
        if (default_tone(1.0) - 1.0).abs() > 1e-4 {
            panic!("expected 1 at 1");
        }
    }

    #[test]
    fn default_tone_bounded() {
        let mut x = -0.5f32;
        while x < 3.0 {
            let y = default_tone(x);
            if !(0.0..=1.0).contains(&y) {
                panic!("out of bounds at x={x}: {y}");
            }
            x += 0.05;
        }
    }

    #[test]
    fn default_tone_monotonic() {
        let mut prev = default_tone(0.0);
        let mut x = 0.01f32;
        while x <= 1.0 {
            let y = default_tone(x);
            if y < prev - 1e-5 {
                panic!("non-monotonic at x={x}: {prev} -> {y}");
            }
            prev = y;
            x += 0.01;
        }
    }
}
