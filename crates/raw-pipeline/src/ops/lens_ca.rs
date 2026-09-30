use super::LinearImage;
use super::lens_correction::{LensCorrection, apply_lens_correction};
use super::{Op, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::{Edits, LensEdits};

pub struct LensCaOp;

impl Op for LensCaOp {
    fn id(&self) -> &'static str {
        "lens_ca"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Pass(super::GpuPass::Lens)
    }
    fn stage(&self) -> Stage {
        Stage::Sensor
    }
    fn order(&self) -> i32 {
        2
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.lens.ca_active()
    }
    fn to_doc(&self, _edits: &Edits) -> Option<serde_json::Value> {
        None
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let (red, blue) = ca_scales(&edits.lens);
        let part = LensCorrection {
            ca: [red, blue],
            ..Default::default()
        };
        apply_lens_correction(image, &part);
        Ok(())
    }
}

pub fn ca_scales(lens: &LensEdits) -> (f32, f32) {
    if !lens.ca_enabled {
        return (1.0, 1.0);
    }
    let (r, b) = lens.ca_scales();
    (r as f32, b as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::PreviewMode;
    use crate::ops::{OpScratch, RenderContext};

    fn ctx() -> OpContext {
        OpContext {
            render: RenderContext {
                wb_coeffs: [1.0; 4],
                cam_to_srgb: crate::color::identity_3x3(),
                is_raw: false,
                capture_sigma: None,
                preview_mode: PreviewMode::None,
                roi: None,
                dcp: None,
            },
            scratch: OpScratch::default(),
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
