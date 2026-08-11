use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;
use crate::tone::shared::{HL_RECONSTRUCT_BIAS, HL_RECONSTRUCT_KNEE, RAW_SENSOR_WHITE};
use std::sync::LazyLock;

pub struct WhiteBalanceOp;

static WB_WGSL: LazyLock<String> = LazyLock::new(|| {
    format!(
        include_str!("../../assets/shaders/ops/white_balance.wgsl"),
        knee = HL_RECONSTRUCT_KNEE,
        white = RAW_SENSOR_WHITE,
        bias = HL_RECONSTRUCT_BIAS,
    )
});

fn camera_wb(raw: [f32; 4]) -> [f32; 4] {
    let mut c = raw;
    if c[0] == 0.0 && c[1] == 0.0 && c[2] == 0.0 {
        c = [1.0, 1.0, 1.0, 1.0];
    }
    if c[1] > 0.0 {
        c[0] /= c[1];
        c[2] /= c[1];
        c[3] /= c[1];
        c[1] = 1.0;
    }
    c
}

impl Op for WhiteBalanceOp {
    fn id(&self) -> &'static str {
        "camera_wb"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
    }
    fn stage(&self) -> Stage {
        Stage::WhiteBalance
    }
    fn is_active(&self, _edits: &Edits) -> bool {
        true
    }
    fn cpu_fused(&self, _edits: &Edits, ctx: &OpContext) -> Option<CpuFusedOp> {
        let c = camera_wb(ctx.render.wb_coeffs);
        Some(CpuFusedOp::WhiteBalance {
            coeffs: [c[0], c[1], c[2]],
            reconstruct: ctx.render.is_raw,
        })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "white_balance",
            WB_WGSL.as_str(),
            "lin = white_balance_apply(lin, p.white_balance);",
        ))
    }
    fn write_gpu_uniform(&self, _edits: &Edits, ctx: &OpContext, dst: &mut [f32]) {
        let c = camera_wb(ctx.render.wb_coeffs);
        dst[0] = c[0];
        dst[1] = c[1];
        dst[2] = c[2];
        dst[3] = if ctx.render.is_raw { 1.0 } else { 0.0 };
    }
}
