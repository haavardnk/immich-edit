use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;
use rayon::prelude::*;

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

impl EditOperator for WhiteBalanceOp {
    fn id(&self) -> &'static str {
        "camera_wb"
    }
    fn stage(&self) -> Stage {
        Stage::WhiteBalance
    }
    fn is_active(&self, _edits: &Edits) -> bool {
        true
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        ctx: &OpContext,
        _edits: &Edits,
    ) -> PipelineResult<()> {
        let coeffs = camera_wb(ctx.wb_coeffs);
        image.rgb.par_chunks_exact_mut(3).for_each(|px| {
            px[0] *= coeffs[0];
            px[1] *= coeffs[1];
            px[2] *= coeffs[2];
        });
        Ok(())
    }
    fn cpu_fused(&self, _edits: &Edits, ctx: &OpContext) -> Option<CpuFusedOp> {
        let c = camera_wb(ctx.wb_coeffs);
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
        let c = camera_wb(ctx.wb_coeffs);
        dst[0] = c[0];
        dst[1] = c[1];
        dst[2] = c[2];
        dst[3] = 1.0;
    }
}
