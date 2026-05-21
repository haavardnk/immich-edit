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
        let d = demosaic::bilinear(&frame.data, frame.width, frame.height, &frame.cfa_pattern);
        (d, frame.width, frame.height)
    } else {
        (frame.data.clone(), frame.width, frame.height)
    };

    let (rgb, w, h) = transform::apply_orientation(rgb, src_w, src_h, frame.orientation);

    let mut image = LinearImage::new(rgb, w, h);
    let cam_to_srgb = if frame.is_raw {
        crate::color::cam_to_srgb_matrix(frame.cam_to_xyz)
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
    apply_baseline_tone(&mut srgb);
    linear_to_srgb(&mut srgb);

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

const HIGHLIGHT_KNEE: f32 = 0.85;
const S_CURVE_AMOUNT: f32 = 0.04;

pub(crate) fn highlight_rolloff(v: f32) -> f32 {
    if v <= HIGHLIGHT_KNEE {
        return v;
    }
    let headroom = 1.0 - HIGHLIGHT_KNEE;
    1.0 - headroom * (-(v - HIGHLIGHT_KNEE) / headroom).exp()
}

pub(crate) fn baseline_tone(v: f32) -> f32 {
    let v = v.max(0.0);
    let shoulder = highlight_rolloff(v);
    let s = shoulder - S_CURVE_AMOUNT * (2.0 * std::f32::consts::PI * shoulder).sin();
    s.clamp(0.0, 1.0)
}

fn apply_baseline_tone(rgb: &mut [f32]) {
    rgb.par_iter_mut().for_each(|v| *v = baseline_tone(*v));
}

fn linear_to_srgb(rgb: &mut [f32]) {
    rgb.par_iter_mut().for_each(|v| {
        *v = if *v <= 0.0031308 {
            *v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolloff_identity_below_knee() {
        if (highlight_rolloff(0.5) - 0.5).abs() >= 1e-6 {
            panic!("expected identity below knee");
        }
        if (highlight_rolloff(HIGHLIGHT_KNEE) - HIGHLIGHT_KNEE).abs() >= 1e-6 {
            panic!("expected identity at knee");
        }
    }

    #[test]
    fn rolloff_bounded_above_knee() {
        for x in [1.0, 1.5, 2.5, 5.0, 100.0] {
            let y = highlight_rolloff(x);
            if !(y > HIGHLIGHT_KNEE && y <= 1.0) {
                panic!("out of bounds at x={x}: {y}");
            }
        }
    }

    #[test]
    fn rolloff_monotonic() {
        let mut prev = -1.0f32;
        let mut x = 0.0f32;
        while x < 3.0 {
            let y = highlight_rolloff(x);
            if y < prev - 1e-6 {
                panic!("non-monotonic at x={x}: {prev} -> {y}");
            }
            prev = y;
            x += 0.05;
        }
    }

    #[test]
    fn baseline_tone_preserves_endpoints() {
        if baseline_tone(0.0).abs() > 1e-4 {
            panic!("expected 0 at 0, got {}", baseline_tone(0.0));
        }
        if (baseline_tone(1.0) - HIGHLIGHT_KNEE).abs() > (1.0 - HIGHLIGHT_KNEE) + 1e-4 {
            panic!("unexpected at 1.0: {}", baseline_tone(1.0));
        }
    }

    #[test]
    fn baseline_tone_bounded() {
        let mut x = -0.5f32;
        while x < 3.0 {
            let y = baseline_tone(x);
            if !(0.0..=1.0).contains(&y) {
                panic!("out of bounds at x={x}: {y}");
            }
            x += 0.05;
        }
    }
}
