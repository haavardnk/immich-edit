use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;
use rayon::prelude::*;

pub struct ToneRegionsOp;

#[inline(always)]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(crate) fn whites_gain(wh: f32) -> f32 {
    1.0 / (1.0 - wh.clamp(-0.99, 0.99) * 0.25)
}

pub(crate) fn highlights_apply(x: f32, hl: f32) -> f32 {
    if hl == 0.0 {
        return x;
    }
    let mask = smoothstep(0.3, 0.95, (x * 1.5).tanh());
    let new = if hl < 0.0 {
        let gamma = 1.0 - hl * 1.75;
        let base = x.clamp(0.0, 1.0).powf(gamma);
        let excess = (x - 1.0).max(0.0);
        let blend = (1.0 + hl).max(0.0);
        base + excess * blend
    } else {
        x * (hl * 1.75).exp2()
    };
    x * (1.0 - mask) + new * mask
}

fn apply_zone(x: f32, hl: f32, sh: f32, bk: f32) -> f32 {
    let x_hi = highlights_apply(x, hl);
    let xc = x_hi.clamp(0.0, 2.0);
    let xm = xc.min(1.0);
    let w_sh = (1.0 - (xm - 0.25).abs() / 0.4).clamp(0.0, 1.0);
    let mut mask_bk = (1.0 - xc / 0.05).clamp(0.0, 1.0);
    mask_bk *= mask_bk;
    let mult_bk = (bk * 0.75).exp2().clamp(0.0, 3.9);
    let delta_bk = xc * (mult_bk - 1.0) * mask_bk;
    let delta = sh * w_sh * xc * 0.5 + delta_bk;
    xc + delta
}

impl EditOperator for ToneRegionsOp {
    fn id(&self) -> &'static str {
        "tone_regions"
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        10
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.tone.highlights != 0.0
            || edits.tone.shadows != 0.0
            || edits.tone.blacks != 0.0
            || edits.tone.whites != 0.0
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let hl = edits.tone.highlights as f32 / 100.0;
        let sh = edits.tone.shadows as f32 / 100.0;
        let bk = edits.tone.blacks as f32 / 100.0;
        let wh = edits.tone.whites as f32 / 100.0;
        let gain = whites_gain(wh);
        image
            .rgb
            .par_iter_mut()
            .for_each(|v| *v = apply_zone(*v * gain, hl, sh, bk));
        Ok(())
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        Some(CpuFusedOp::ToneRegions {
            hl: edits.tone.highlights as f32 / 100.0,
            sh: edits.tone.shadows as f32 / 100.0,
            bk: edits.tone.blacks as f32 / 100.0,
            wh_gain: whites_gain(edits.tone.whites as f32 / 100.0),
        })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "tone_regions",
            "fn tone_regions_whites_gain(wh: f32) -> f32 { return 1.0 / (1.0 - clamp(wh, -0.99, 0.99) * 0.25); } fn tone_regions_highlights(x: f32, hl: f32) -> f32 { if (hl == 0.0) { return x; } let mask = smoothstep(0.3, 0.95, tanh(x * 1.5)); var new_v: f32; if (hl < 0.0) { let gamma = 1.0 - hl * 1.75; let base = pow(max(min(x, 1.0), 0.0), gamma); let excess = max(x - 1.0, 0.0); let blend = max(1.0 + hl, 0.0); new_v = base + excess * blend; } else { new_v = x * exp2(hl * 1.75); } return x * (1.0 - mask) + new_v * mask; } fn tone_regions_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> { if (p.x == 0.0 && p.y == 0.0 && p.z == 0.0 && p.w == 0.0) { return c; } let gain = tone_regions_whites_gain(p.w); var out_v = vec3<f32>(0.0); for (var i = 0u; i < 3u; i = i + 1u) { let x_hi = tone_regions_highlights(c[i] * gain, p.x); let xc = clamp(x_hi, 0.0, 2.0); let xm = min(xc, 1.0); let w_sh = clamp(1.0 - abs(xm - 0.25) / 0.4, 0.0, 1.0); var mask_bk = clamp(1.0 - xc / 0.05, 0.0, 1.0); mask_bk = mask_bk * mask_bk; let mult_bk = clamp(exp2(p.z * 0.75), 0.0, 3.9); let delta_bk = xc * (mult_bk - 1.0) * mask_bk; let delta = p.y * w_sh * xc * 0.5 + delta_bk; out_v[i] = xc + delta; } return out_v; }",
            "lin = tone_regions_apply(lin, p.tone_regions);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = edits.tone.highlights as f32 / 100.0;
        dst[1] = edits.tone.shadows as f32 / 100.0;
        dst[2] = edits.tone.blacks as f32 / 100.0;
        dst[3] = edits.tone.whites as f32 / 100.0;
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if !self.is_active(edits) {
            return None;
        }
        Some(serde_json::json!({
            "highlights": edits.tone.highlights,
            "shadows": edits.tone.shadows,
            "blacks": edits.tone.blacks,
            "whites": edits.tone.whites,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("highlights").and_then(|v| v.as_f64()) {
            edits.tone.highlights = v;
        }
        if let Some(v) = value.get("shadows").and_then(|v| v.as_f64()) {
            edits.tone.shadows = v;
        }
        if let Some(v) = value.get("blacks").and_then(|v| v.as_f64()) {
            edits.tone.blacks = v;
        }
        if let Some(v) = value.get("whites").and_then(|v| v.as_f64()) {
            edits.tone.whites = v;
        }
    }
}
