use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::fused::CpuFusedOp;
use crate::cpu::presence_pyramid::LumaPyramid;
use crate::edits::Edits;
use crate::presence::{presence_mips, presence_pyramid_levels, presence_radii};
use rayon::prelude::*;
use std::sync::Arc;

pub struct ToneRegionsOp;

#[inline(always)]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(crate) fn whites_gain(wh: f32) -> f32 {
    1.0 / (1.0 - wh.clamp(-0.99, 0.99) * 0.5)
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

pub(crate) fn shadows_mult(luma: f32, blur_l: f32, sh: f32) -> f32 {
    if sh == 0.0 {
        return 1.0;
    }
    let mut mask = (1.0 - luma / 0.25).clamp(0.0, 1.0);
    mask *= mask;
    let edge = (luma.max(0.0).sqrt() - blur_l.max(0.0).sqrt()).abs();
    let halo = 1.0 - smoothstep(0.05, 0.25, edge);
    let mult = (sh * 1.5 * halo).exp2().clamp(0.1, 3.9);
    1.0 + (mult - 1.0) * mask
}

#[inline(always)]
fn blacks_scalar(x: f32, bk: f32) -> f32 {
    let xc = x.clamp(0.0, 2.0);
    let mut mask_bk = (1.0 - xc / 0.1).clamp(0.0, 1.0);
    mask_bk *= mask_bk;
    let mult_bk = (bk * 1.5).exp2().clamp(0.0, 3.9);
    xc + xc * (mult_bk - 1.0) * mask_bk
}

pub(crate) fn apply_tone_regions_rgb(r: f32, g: f32, b: f32, hl: f32, bk: f32) -> (f32, f32, f32) {
    let clip = (r.max(g).max(b) - 1.0).max(0.0);
    let mut rr = highlights_apply(r, hl);
    let mut gg = highlights_apply(g, hl);
    let mut bb = highlights_apply(b, hl);
    let desat = smoothstep(0.0, 0.35, clip) * (-hl).clamp(0.0, 1.0);
    if desat > 0.0 {
        let luma = 0.2126 * rr + 0.7152 * gg + 0.0722 * bb;
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
        ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let hl = edits.tone.highlights as f32 / 100.0;
        let sh = edits.tone.shadows as f32 / 100.0;
        let bk = edits.tone.blacks as f32 / 100.0;
        let wh = edits.tone.whites as f32 / 100.0;
        let gain = whites_gain(wh);
        if sh == 0.0 {
            image.rgb.par_chunks_exact_mut(3).for_each(|px| {
                let (nr, ng, nb) =
                    apply_tone_regions_rgb(px[0] * gain, px[1] * gain, px[2] * gain, hl, bk);
                px[0] = nr;
                px[1] = ng;
                px[2] = nb;
            });
            return Ok(());
        }
        let blur_arc = ctx.shadows_blur.clone().unwrap_or_else(|| {
            let w = image.width as u32;
            let h = image.height as u32;
            let radii = presence_radii(w, h);
            let mips = presence_mips(w, h, radii);
            let levels = presence_pyramid_levels(w, h, radii) as usize;
            let pyr = LumaPyramid::build(image, levels);
            Arc::new(pyr.upsample(mips.shadows, image.width, image.height))
        });
        let blur = blur_arc.as_slice();
        image
            .rgb
            .par_chunks_exact_mut(3)
            .enumerate()
            .for_each(|(i, px)| {
                px[0] *= gain;
                px[1] *= gain;
                px[2] *= gain;
                let luma = 0.2126 * px[0] + 0.7152 * px[1] + 0.0722 * px[2];
                let mult = shadows_mult(luma, blur[i], sh);
                px[0] *= mult;
                px[1] *= mult;
                px[2] *= mult;
                let (nr, ng, nb) = apply_tone_regions_rgb(px[0], px[1], px[2], hl, bk);
                px[0] = nr;
                px[1] = ng;
                px[2] = nb;
            });
        Ok(())
    }
    fn cpu_fused(&self, edits: &Edits, ctx: &OpContext) -> Option<CpuFusedOp> {
        Some(CpuFusedOp::ToneRegions {
            hl: edits.tone.highlights as f32 / 100.0,
            sh: edits.tone.shadows as f32 / 100.0,
            bk: edits.tone.blacks as f32 / 100.0,
            wh_gain: whites_gain(edits.tone.whites as f32 / 100.0),
            shadows_blur: ctx.shadows_blur.clone(),
        })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "tone_regions",
            TONE_REGIONS_WGSL,
            "lin = tone_regions_apply(lin, p.tone_regions, shadows_blur_l);",
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

const TONE_REGIONS_WGSL: &str = r#"
fn tone_regions_whites_gain(wh: f32) -> f32 {
    return 1.0 / (1.0 - clamp(wh, -0.99, 0.99) * 0.5);
}
fn tone_regions_highlights(x: f32, hl: f32) -> f32 {
    if (hl == 0.0) { return x; }
    let mask = smoothstep(0.3, 0.95, tanh(x * 1.5));
    var new_v: f32;
    if (hl < 0.0) {
        let gamma = 1.0 - hl * 1.75;
        let base = pow(max(min(x, 1.0), 0.0), gamma);
        let excess = max(x - 1.0, 0.0);
        let blend = max(1.0 + hl, 0.0);
        new_v = base + excess * blend;
    } else {
        new_v = x * exp2(hl * 1.75);
    }
    return x * (1.0 - mask) + new_v * mask;
}
fn tone_regions_shadows_mult(luma: f32, blur_l: f32, sh: f32) -> f32 {
    if (sh == 0.0) { return 1.0; }
    var mask = clamp(1.0 - luma / 0.25, 0.0, 1.0);
    mask = mask * mask;
    let edge = abs(sqrt(max(luma, 0.0)) - sqrt(max(blur_l, 0.0)));
    let halo = 1.0 - smoothstep(0.05, 0.25, edge);
    let mult = clamp(exp2(sh * 1.5 * halo), 0.1, 3.9);
    return 1.0 + (mult - 1.0) * mask;
}
fn tone_regions_blacks(x: f32, bk: f32) -> f32 {
    let xc = clamp(x, 0.0, 2.0);
    var mask_bk = clamp(1.0 - xc / 0.1, 0.0, 1.0);
    mask_bk = mask_bk * mask_bk;
    let mult_bk = clamp(exp2(bk * 1.5), 0.0, 3.9);
    return xc + xc * (mult_bk - 1.0) * mask_bk;
}
fn tone_regions_apply_rgb(c: vec3<f32>, hl: f32, bk: f32) -> vec3<f32> {
    let clip = max(max(max(c.r, c.g), c.b) - 1.0, 0.0);
    var v = vec3<f32>(
        tone_regions_highlights(c.r, hl),
        tone_regions_highlights(c.g, hl),
        tone_regions_highlights(c.b, hl)
    );
    let desat = smoothstep(0.0, 0.35, clip) * clamp(-hl, 0.0, 1.0);
    if (desat > 0.0) {
        let luma = 0.2126 * v.x + 0.7152 * v.y + 0.0722 * v.z;
        v = mix(v, vec3<f32>(luma), desat);
    }
    return vec3<f32>(
        tone_regions_blacks(v.x, bk),
        tone_regions_blacks(v.y, bk),
        tone_regions_blacks(v.z, bk)
    );
}
fn tone_regions_apply(c: vec3<f32>, p: vec4<f32>, blur_l: f32) -> vec3<f32> {
    if (p.x == 0.0 && p.y == 0.0 && p.z == 0.0 && p.w == 0.0) { return c; }
    let gain = tone_regions_whites_gain(p.w);
    var v = c * gain;
    let luma = 0.2126 * v.x + 0.7152 * v.y + 0.0722 * v.z;
    let sm = tone_regions_shadows_mult(luma, blur_l, p.y);
    v = v * sm;
    return tone_regions_apply_rgb(v, p.x, p.z);
}
"#;
