use super::LinearImage;
use super::sample::sample_channel_bicubic;
use super::{EditOperator, OpContext, Stage, OpKind};
use crate::PipelineResult;
use crate::edits::{Edits, LensEdits};
use rayon::prelude::*;

pub struct LensCaOp;

impl EditOperator for LensCaOp {
    fn id(&self) -> &'static str {
        "lens_ca"
    }
    fn stage(&self) -> Stage {
        Stage::Sensor
    }
    fn kind(&self) -> OpKind {
        OpKind::Spatial
    }
    fn order(&self) -> i32 {
        2
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.lens.ca_active()
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        apply_lens_ca(image, &edits.lens);
        Ok(())
    }
    fn to_doc(&self, _edits: &Edits) -> Option<serde_json::Value> {
        None
    }
}

pub fn ca_scales(lens: &LensEdits) -> (f32, f32) {
    if !lens.ca_enabled {
        return (1.0, 1.0);
    }
    let (r, b) = lens.ca_scales();
    (r as f32, b as f32)
}

pub fn apply_lens_ca(image: &mut LinearImage, lens: &LensEdits) {
    let w = image.width;
    let h = image.height;
    if w == 0 || h == 0 {
        return;
    }
    let (red_scale, blue_scale) = ca_scales(lens);
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;
    let src = image.rgb.clone();
    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let dy = y as f32 + 0.5 - cy;
            for x in 0..w {
                let dx = x as f32 + 0.5 - cx;
                let rx = cx + dx * red_scale - 0.5;
                let ry = cy + dy * red_scale - 0.5;
                let bx = cx + dx * blue_scale - 0.5;
                let by = cy + dy * blue_scale - 0.5;
                let i = x * 3;
                row[i] = sample_channel_bicubic(&src, w, h, rx, ry, 0);
                row[i + 2] = sample_channel_bicubic(&src, w, h, bx, by, 2);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::PreviewMode;

    fn ctx() -> OpContext {
        OpContext {
            wb_coeffs: [1.0; 4],
            cam_to_srgb: crate::color::identity_3x3(),
            is_raw: false,
            preview_mode: PreviewMode::None,
            shadows_blur: None,
        }
    }

    #[test]
    fn zero_is_identity() {
        let w = 32;
        let h = 24;
        let mut rgb = vec![0.0f32; w * h * 3];
        for (i, v) in rgb.iter_mut().enumerate() {
            *v = (i as f32 * 0.001).fract();
        }
        let before = rgb.clone();
        let mut img = LinearImage::new(rgb, w, h);
        LensCaOp
            .apply_cpu(&mut img, &ctx(), &Edits::default())
            .unwrap();
        for (a, b) in img.rgb.iter().zip(before.iter()) {
            if (a - b).abs() > 1e-6 {
                panic!("expected identity");
            }
        }
    }

    #[test]
    fn shifts_red_channel_only() {
        let w = 64;
        let h = 48;
        let mut rgb = vec![0.0f32; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) * 3;
                let v = (x as f32) / (w as f32 - 1.0);
                rgb[i] = v;
                rgb[i + 1] = v;
                rgb[i + 2] = v;
            }
        }
        let mut img = LinearImage::new(rgb, w, h);
        let edits = Edits {
            lens: LensEdits {
                ca_enabled: true,
                ca_red_scale_x10000: 100.0,
                ..Default::default()
            },
            ..Default::default()
        };
        LensCaOp.apply_cpu(&mut img, &ctx(), &edits).unwrap();
        let idx = (10 * w + 5) * 3;
        if (img.rgb[idx + 1] - img.rgb[idx + 2]).abs() > 1e-5 {
            panic!("green and blue should be untouched");
        }
        if (img.rgb[idx] - img.rgb[idx + 1]).abs() < 1e-5 {
            panic!("red should differ from green after CA shift");
        }
    }
}
