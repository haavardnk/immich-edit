use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;
use crate::math::{luma, smoothstep};

pub struct ToneRegionsOp;

pub const TONE_REGIONS_WHITES_CLAMP: f32 = 0.99;
pub const TONE_REGIONS_WHITES_SCALE: f32 = 0.5;
pub const TONE_REGIONS_HL_MASK_LO: f32 = 0.3;
pub const TONE_REGIONS_HL_MASK_HI: f32 = 0.95;
pub const TONE_REGIONS_HL_MASK_TANH: f32 = 1.5;
pub const TONE_REGIONS_HL_STRENGTH: f32 = 1.75;
pub const TONE_REGIONS_HL_DESAT_LO: f32 = 0.0;
pub const TONE_REGIONS_HL_DESAT_HI: f32 = 0.35;
pub const TONE_REGIONS_SH_MASK_RANGE: f32 = 0.25;
pub const TONE_REGIONS_SH_HALO_LO: f32 = 0.05;
pub const TONE_REGIONS_SH_HALO_HI: f32 = 0.25;
pub const TONE_REGIONS_SH_STRENGTH: f32 = 1.5;
pub const TONE_REGIONS_SH_MULT_MIN: f32 = 0.1;
pub const TONE_REGIONS_SH_MULT_MAX: f32 = 3.9;
pub const TONE_REGIONS_BK_CEILING: f32 = 2.0;
pub const TONE_REGIONS_BK_MASK_RANGE: f32 = 0.1;
pub const TONE_REGIONS_BK_STRENGTH: f32 = 1.5;
pub const TONE_REGIONS_BK_MULT_MAX: f32 = 3.9;

pub(crate) fn whites_gain(wh: f32) -> f32 {
    1.0 / (1.0
        - wh.clamp(-TONE_REGIONS_WHITES_CLAMP, TONE_REGIONS_WHITES_CLAMP)
            * TONE_REGIONS_WHITES_SCALE)
}

pub(crate) fn highlights_apply(x: f32, hl: f32) -> f32 {
    if hl == 0.0 {
        return x;
    }
    let mask = smoothstep(
        TONE_REGIONS_HL_MASK_LO,
        TONE_REGIONS_HL_MASK_HI,
        (x * TONE_REGIONS_HL_MASK_TANH).tanh(),
    );
    let new = if hl < 0.0 {
        let gamma = 1.0 - hl * TONE_REGIONS_HL_STRENGTH;
        let base = x.clamp(0.0, 1.0).powf(gamma);
        let excess = (x - 1.0).max(0.0);
        let blend = (1.0 + hl).max(0.0);
        base + excess * blend
    } else {
        x * (hl * TONE_REGIONS_HL_STRENGTH).exp2()
    };
    x * (1.0 - mask) + new * mask
}

pub(crate) fn shadows_mult(luma: f32, blur_l: f32, sh: f32) -> f32 {
    if sh == 0.0 {
        return 1.0;
    }
    let mut mask = (1.0 - luma / TONE_REGIONS_SH_MASK_RANGE).clamp(0.0, 1.0);
    mask *= mask;
    let edge = (luma.max(0.0).sqrt() - blur_l.max(0.0).sqrt()).abs();
    let halo = 1.0 - smoothstep(TONE_REGIONS_SH_HALO_LO, TONE_REGIONS_SH_HALO_HI, edge);
    let mult = (sh * TONE_REGIONS_SH_STRENGTH * halo)
        .exp2()
        .clamp(TONE_REGIONS_SH_MULT_MIN, TONE_REGIONS_SH_MULT_MAX);
    1.0 + (mult - 1.0) * mask
}

#[inline(always)]
fn blacks_scalar(x: f32, bk: f32) -> f32 {
    let xc = x.clamp(0.0, TONE_REGIONS_BK_CEILING);
    let mut mask_bk = (1.0 - xc / TONE_REGIONS_BK_MASK_RANGE).clamp(0.0, 1.0);
    mask_bk *= mask_bk;
    let mult_bk = (bk * TONE_REGIONS_BK_STRENGTH)
        .exp2()
        .clamp(0.0, TONE_REGIONS_BK_MULT_MAX);
    xc + xc * (mult_bk - 1.0) * mask_bk
}

pub(crate) fn apply_tone_regions_rgb(r: f32, g: f32, b: f32, hl: f32, bk: f32) -> (f32, f32, f32) {
    let clip = (r.max(g).max(b) - 1.0).max(0.0);
    let mut rr = highlights_apply(r, hl);
    let mut gg = highlights_apply(g, hl);
    let mut bb = highlights_apply(b, hl);
    let desat = smoothstep(TONE_REGIONS_HL_DESAT_LO, TONE_REGIONS_HL_DESAT_HI, clip)
        * (-hl).clamp(0.0, 1.0);
    if desat > 0.0 {
        let luma = luma(rr, gg, bb);
        rr = rr + (luma - rr) * desat;
        gg = gg + (luma - gg) * desat;
        bb = bb + (luma - bb) * desat;
    }
    (
        blacks_scalar(rr, bk),
        blacks_scalar(gg, bk),
        blacks_scalar(bb, bk),
    )
}

impl Op for ToneRegionsOp {
    fn id(&self) -> &'static str {
        "tone_regions"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
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
    fn cpu_fused(&self, edits: &Edits, ctx: &OpContext) -> Option<CpuFusedOp> {
        Some(CpuFusedOp::ToneRegions {
            hl: edits.tone.highlights as f32 / 100.0,
            sh: edits.tone.shadows as f32 / 100.0,
            bk: edits.tone.blacks as f32 / 100.0,
            wh_gain: whites_gain(edits.tone.whites as f32 / 100.0),
            shadows_blur: ctx.scratch.shadows_blur.clone(),
        })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "tone_regions",
            include_str!("../../assets/shaders/ops/tone_regions.wgsl"),
            "lin = tone_regions_apply(lin, p.tone_regions, shadows_blur_l);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = edits.tone.highlights as f32 / 100.0;
        dst[1] = edits.tone.shadows as f32 / 100.0;
        dst[2] = edits.tone.blacks as f32 / 100.0;
        dst[3] = edits.tone.whites as f32 / 100.0;
    }
}
