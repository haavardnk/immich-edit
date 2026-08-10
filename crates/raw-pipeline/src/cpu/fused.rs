use crate::edits::HSL_BANDS;
use crate::ops::LinearImage;
use crate::ops::curves::{CurveLuts, apply_curves_pixel};
use rayon::prelude::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum CpuFusedOp {
    WhiteBalance {
        coeffs: [f32; 3],
        reconstruct: bool,
    },
    ColorMatrix {
        m: [[f32; 3]; 3],
    },
    Exposure {
        factor: f32,
    },
    Brightness {
        amount: f32,
    },
    Contrast {
        s: f32,
    },
    Saturation {
        factor: f32,
    },
    Vibrance {
        amount: f32,
    },
    ToneRegions {
        hl: f32,
        sh: f32,
        bk: f32,
        wh_gain: f32,
        shadows_blur: Option<Arc<Vec<f32>>>,
    },
    Curves {
        luts: Box<CurveLuts>,
    },
    Hsl {
        hue_shifts: [f32; HSL_BANDS],
        sat_gains: [f32; HSL_BANDS],
        lum_gains: [f32; HSL_BANDS],
    },
    ColorGrade {
        s_off: [f32; 3],
        s_lum: f32,
        m_off: [f32; 3],
        m_lum: f32,
        h_off: [f32; 3],
        h_lum: f32,
        g_off: [f32; 3],
        g_lum: f32,
        balance: f32,
        blend: f32,
    },
    Presence {
        texture: f32,
        clarity: f32,
        texture_blur: Option<Arc<Vec<f32>>>,
        clarity_blur: Option<Arc<Vec<f32>>>,
    },
    DcpHueSat {
        map: Arc<crate::dcp::HueSatMap>,
        to_pp: [[f32; 3]; 3],
        from_pp: [[f32; 3]; 3],
    },
}

#[derive(Default, Clone, Debug)]
pub struct FusedSegment {
    pub ops: Vec<CpuFusedOp>,
}

impl FusedSegment {
    pub fn push(&mut self, op: CpuFusedOp) {
        self.ops.push(op);
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn clear(&mut self) {
        self.ops.clear();
    }
}

pub mod color_grade;
pub mod hsl;
pub mod presence;

#[inline(always)]
pub fn apply_one(op: &CpuFusedOp, i: usize, r: &mut f32, g: &mut f32, b: &mut f32) {
    match op {
        CpuFusedOp::WhiteBalance {
            coeffs,
            reconstruct,
        } => presence::apply_white_balance(coeffs, *reconstruct, r, g, b),
        CpuFusedOp::ColorMatrix { m } => {
            let nr = m[0][0] * *r + m[0][1] * *g + m[0][2] * *b;
            let ng = m[1][0] * *r + m[1][1] * *g + m[1][2] * *b;
            let nb = m[2][0] * *r + m[2][1] * *g + m[2][2] * *b;
            *r = nr;
            *g = ng;
            *b = nb;
        }
        CpuFusedOp::Exposure { factor } => {
            *r *= *factor;
            *g *= *factor;
            *b *= *factor;
        }
        CpuFusedOp::Brightness { amount } => {
            let (nr, ng, nb) = crate::ops::brightness::apply_brightness_rgb(*r, *g, *b, *amount);
            *r = nr;
            *g = ng;
            *b = nb;
        }
        CpuFusedOp::Contrast { s } => {
            *r = crate::ops::contrast::apply_perceptual_contrast(*r, *s);
            *g = crate::ops::contrast::apply_perceptual_contrast(*g, *s);
            *b = crate::ops::contrast::apply_perceptual_contrast(*b, *s);
        }
        CpuFusedOp::Saturation { factor } => {
            let luma = 0.2126 * *r + 0.7152 * *g + 0.0722 * *b;
            *r = luma + (*r - luma) * *factor;
            *g = luma + (*g - luma) * *factor;
            *b = luma + (*b - luma) * *factor;
        }
        CpuFusedOp::Vibrance { amount } => {
            let (nr, ng, nb) = crate::ops::vibrance::apply_vibrance_rgb(*r, *g, *b, *amount);
            *r = nr;
            *g = ng;
            *b = nb;
        }
        CpuFusedOp::ToneRegions {
            hl,
            sh,
            bk,
            wh_gain,
            shadows_blur,
        } => {
            *r *= *wh_gain;
            *g *= *wh_gain;
            *b *= *wh_gain;
            if *sh != 0.0 {
                let luma = 0.2126 * *r + 0.7152 * *g + 0.0722 * *b;
                let blur_l = shadows_blur.as_ref().map(|buf| buf[i]).unwrap_or(luma);
                let mult = crate::ops::tone_regions::shadows_mult(luma, blur_l, *sh);
                *r *= mult;
                *g *= mult;
                *b *= mult;
            }
            let (nr, ng, nb) =
                crate::ops::tone_regions::apply_tone_regions_rgb(*r, *g, *b, *hl, *bk);
            *r = nr;
            *g = ng;
            *b = nb;
        }
        CpuFusedOp::Curves { luts } => apply_curves_pixel(luts.as_ref(), r, g, b),
        CpuFusedOp::Hsl {
            hue_shifts,
            sat_gains,
            lum_gains,
        } => hsl::apply_hsl(hue_shifts, sat_gains, lum_gains, r, g, b),
        CpuFusedOp::ColorGrade {
            s_off,
            s_lum,
            m_off,
            m_lum,
            h_off,
            h_lum,
            g_off,
            g_lum,
            balance,
            blend,
        } => color_grade::apply_color_grade(
            color_grade::ColorGradeParams {
                s_off,
                s_lum: *s_lum,
                m_off,
                m_lum: *m_lum,
                h_off,
                h_lum: *h_lum,
                g_off,
                g_lum: *g_lum,
                balance: *balance,
                blend: *blend,
            },
            r,
            g,
            b,
        ),
        CpuFusedOp::Presence {
            texture,
            clarity,
            texture_blur,
            clarity_blur,
        } => presence::apply_presence(
            presence::PresenceParams {
                texture: *texture,
                clarity: *clarity,
                texture_blur: texture_blur.as_ref(),
                clarity_blur: clarity_blur.as_ref(),
            },
            i,
            r,
            g,
            b,
        ),
        CpuFusedOp::DcpHueSat {
            map,
            to_pp,
            from_pp,
        } => {
            let out = crate::color::apply_huesat(map, to_pp, from_pp, [*r, *g, *b]);
            *r = out[0];
            *g = out[1];
            *b = out[2];
        }
    }
}

pub fn apply_segment(image: &mut LinearImage, segment: &FusedSegment) {
    if segment.is_empty() {
        return;
    }
    let ops = segment.ops.as_slice();
    let img_w = image.width;
    let row_floats = img_w * 3;
    image
        .rgb
        .par_chunks_exact_mut(row_floats)
        .enumerate()
        .for_each(|(y, row)| {
            let row_base = y * img_w;
            for (x, px) in row.chunks_exact_mut(3).enumerate() {
                let i = row_base + x;
                let mut r = px[0];
                let mut g = px[1];
                let mut b = px[2];
                for op in ops {
                    apply_one(op, i, &mut r, &mut g, &mut b);
                }
                px[0] = r;
                px[1] = g;
                px[2] = b;
            }
        });
}

#[cfg(test)]
mod tests {
    use crate::color::identity_3x3;
    use crate::edits::Edits;
    use crate::ops::Op;
    use crate::ops::{OpContext, OpScratch, RenderContext, color_matrix};

    #[test]
    fn fused_skips_color_matrix_when_not_raw() {
        let ctx = OpContext {
            render: RenderContext {
                wb_coeffs: [1.0, 1.0, 1.0, 1.0],
                cam_to_srgb: identity_3x3(),
                is_raw: false,
                capture_sigma: None,
                preview_mode: crate::frame::PreviewMode::None,
                dcp: None,
            },
            scratch: OpScratch::default(),
        };
        let edits = Edits::default();
        if color_matrix::ColorMatrixOp
            .cpu_fused(&edits, &ctx)
            .is_some()
        {
            panic!("expected None when not raw");
        }
    }

    fn wb(r: f32, g: f32, b: f32) -> [f32; 3] {
        let op = super::CpuFusedOp::WhiteBalance {
            coeffs: [2.0, 1.0, 1.5],
            reconstruct: true,
        };
        let mut rr = r;
        let mut gg = g;
        let mut bb = b;
        super::apply_one(&op, 0, &mut rr, &mut gg, &mut bb);
        [rr, gg, bb]
    }

    #[test]
    fn wb_keeps_unclipped_blue_blue() {
        let out = wb(0.2, 0.4, 0.8);
        if (out[0] - 0.4).abs() > 1e-5 || (out[1] - 0.4).abs() > 1e-5 || (out[2] - 1.2).abs() > 1e-5
        {
            panic!("unclipped pixel should be plain WB multiply, got {out:?}");
        }
        if !(out[2] > out[1] && out[1] >= out[0]) {
            panic!("blue should remain dominant: {out:?}");
        }
    }

    #[test]
    fn wb_deep_blue_below_knee_is_untouched() {
        let out = wb(0.221, 0.711, 0.564);
        let plain = [0.442f32, 0.711, 0.846];
        for c in 0..3 {
            if (out[c] - plain[c]).abs() > 1e-4 {
                panic!("deep blue (max green 0.71 < knee) must be plain WB multiply, got {out:?}");
            }
        }
    }

    #[test]
    fn wb_green_clipped_reduces_magenta() {
        let pre = [0.546f32, 1.034, 0.948];
        let coeffs = [2.0f32, 1.0, 1.5];
        let plain = [pre[0] * coeffs[0], pre[1] * coeffs[1], pre[2] * coeffs[2]];
        let plain_deficit = (plain[0] + plain[2]) * 0.5 - plain[1];
        let out = wb(pre[0], pre[1], pre[2]);
        let out_deficit = (out[0] + out[2]) * 0.5 - out[1];
        if out_deficit >= plain_deficit - 1e-4 {
            panic!("green-clip reconstruction must lift green to cut magenta, got {out:?}");
        }
        if out[1] < plain[1] - 1e-4 {
            panic!("reconstruction must not lower the clipped green channel, got {out:?}");
        }
    }

    #[test]
    fn wb_saturated_color_max_channel_stays_dominant() {
        let out = wb(0.3, 0.3, 0.99);
        if !(out[2] > out[0] && out[2] > out[1]) {
            panic!("a pixel whose own max channel is clipped must stay that color, got {out:?}");
        }
    }

    #[test]
    fn wb_reconstruction_never_pushes_below_plain() {
        let out = wb(1.0, 1.0, 1.0);
        let plain = [2.0f32, 1.0, 1.5];
        let plain_min = plain[0].min(plain[1]).min(plain[2]);
        let out_min = out[0].min(out[1]).min(out[2]);
        if out_min < plain_min - 1e-4 {
            panic!(
                "reconstruction toward wmax must never push a channel below plain WB, got {out:?}"
            );
        }
        if (out[0] - out[1]).abs() > 1e-4 || (out[1] - out[2]).abs() > 1e-4 {
            panic!("fully clipped neutral should be neutral after reconstruction, got {out:?}");
        }
    }
}
