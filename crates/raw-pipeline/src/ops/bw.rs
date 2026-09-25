use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::{BwEdits, BwTint, Edits};
use crate::math::luma;

pub const BW_MIX_EV: f32 = 1.5;
pub const BW_TINT_STRENGTH: f32 = 0.5;
pub const BW_TINT_PIVOT: f32 = 0.5;
pub const BW_TINT_BALANCE_RANGE: f32 = 0.3;
pub const BW_TINT_WIDTH: f32 = 0.35;

pub struct BwOp;

fn tint(tint: &BwTint) -> [f32; 3] {
    let dir = super::color_grade::hue_dir(tint.hue as f32);
    let neutral = luma(dir[0], dir[1], dir[2]);
    let gain = BW_TINT_STRENGTH * tint.sat as f32 / 100.0;
    dir.map(|c| (c - neutral) * gain)
}

fn mix(bw: &BwEdits) -> [f32; 6] {
    bw.mix.channels().map(|v| v as f32 / 100.0)
}

impl Op for BwOp {
    fn id(&self) -> &'static str {
        "bw"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
    }
    fn stage(&self) -> Stage {
        Stage::Color
    }
    fn order(&self) -> i32 {
        10
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.color.bw.enabled
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let bw = &edits.color.bw;
        if bw.is_neutral() {
            return None;
        }
        serde_json::to_value(bw).ok()
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Ok(bw) = serde_json::from_value(value.clone()) {
            edits.color.bw = bw;
        }
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "bw",
            functions: include_str!("../../assets/shaders/ops/bw.wgsl"),
            apply: "lin = bw_apply(lin);",
            vec4_count: 4,
        })
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        let bw = &edits.color.bw;
        Some(CpuFusedOp::Bw {
            mix: mix(bw),
            shadow_tint: tint(&bw.shadows),
            highlight_tint: tint(&bw.highlights),
            balance: bw.balance as f32 / 100.0,
        })
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        let bw = &edits.color.bw;
        let mix = mix(bw);
        let shadow = tint(&bw.shadows);
        let highlight = tint(&bw.highlights);
        dst[..6].copy_from_slice(&mix);
        dst[6] = 0.0;
        dst[7] = 0.0;
        dst[8..11].copy_from_slice(&shadow);
        dst[11] = bw.balance as f32 / 100.0;
        dst[12..15].copy_from_slice(&highlight);
        dst[15] = 0.0;
    }
}
