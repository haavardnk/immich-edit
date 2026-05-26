use super::{FusedOp, GpuOp, OpContext, OpMeta, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;

pub struct WhiteBalanceOp;

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

impl OpMeta for WhiteBalanceOp {
    fn id(&self) -> &'static str {
        "camera_wb"
    }
    fn stage(&self) -> Stage {
        Stage::WhiteBalance
    }
    fn is_active(&self, _edits: &Edits) -> bool {
        true
    }
}

impl FusedOp for WhiteBalanceOp {
    fn cpu_fused(&self, _edits: &Edits, ctx: &OpContext) -> Option<CpuFusedOp> {
        let c = camera_wb(ctx.render.wb_coeffs);
        Some(CpuFusedOp::WhiteBalance {
            coeffs: [c[0], c[1], c[2]],
        })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "white_balance",
            "fn white_balance_apply(c: vec3<f32>, w: vec4<f32>) -> vec3<f32> { return vec3<f32>(c.r * w.r, c.g * w.g, c.b * w.b); }",
            "lin = white_balance_apply(lin, p.white_balance);",
        ))
    }
    fn write_gpu_uniform(&self, _edits: &Edits, ctx: &OpContext, dst: &mut [f32]) {
        let c = camera_wb(ctx.render.wb_coeffs);
        dst[0] = c[0];
        dst[1] = c[1];
        dst[2] = c[2];
        dst[3] = 1.0;
    }
}
