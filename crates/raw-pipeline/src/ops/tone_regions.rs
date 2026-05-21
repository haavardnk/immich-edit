use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::Edits;
use rayon::prelude::*;

pub struct ToneRegionsOp;

fn weights(x: f32) -> [f32; 4] {
    let bk = ((0.2 - x) / 0.2).clamp(0.0, 1.0);
    let sh = (1.0 - (x - 0.25).abs() / 0.4).clamp(0.0, 1.0);
    let hl = (1.0 - (x - 0.75).abs() / 0.4).clamp(0.0, 1.0);
    let wh = ((x - 0.8) / 0.2).clamp(0.0, 1.0);
    [hl, sh, bk, wh]
}

fn apply_zone(x: f32, hl: f32, sh: f32, bk: f32, wh: f32) -> f32 {
    let xc = x.clamp(0.0, 2.0);
    let [w_hl, w_sh, w_bk, w_wh] = weights(xc.min(1.0));
    let delta = hl * w_hl * (1.0 - xc).max(-1.0) * 0.5
        + sh * w_sh * xc * 0.5
        + bk * w_bk * 0.2
        + wh * w_wh * (1.0 - xc).max(-1.0) * 0.5;
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
        image
            .rgb
            .par_iter_mut()
            .for_each(|v| *v = apply_zone(*v, hl, sh, bk, wh));
        Ok(())
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "tone_regions",
            functions: "fn tone_regions_weights(x: f32) -> vec4<f32> { let bk = clamp((0.2 - x) / 0.2, 0.0, 1.0); let sh = clamp(1.0 - abs(x - 0.25) / 0.4, 0.0, 1.0); let hl = clamp(1.0 - abs(x - 0.75) / 0.4, 0.0, 1.0); let wh = clamp((x - 0.8) / 0.2, 0.0, 1.0); return vec4<f32>(hl, sh, bk, wh); } fn tone_regions_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> { if (p.x == 0.0 && p.y == 0.0 && p.z == 0.0 && p.w == 0.0) { return c; } var out_v = vec3<f32>(0.0); for (var i = 0u; i < 3u; i = i + 1u) { let xc = clamp(c[i], 0.0, 2.0); let w = tone_regions_weights(min(xc, 1.0)); let delta = p.x * w.x * max(1.0 - xc, -1.0) * 0.5 + p.y * w.y * xc * 0.5 + p.z * w.z * 0.2 + p.w * w.w * max(1.0 - xc, -1.0) * 0.5; out_v[i] = xc + delta; } return out_v; }",
            apply: "lin = tone_regions_apply(lin, p.tone_regions);",
        })
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32; 4]) {
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
