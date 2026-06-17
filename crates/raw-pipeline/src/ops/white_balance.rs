use super::{FusedOp, GpuOp, OpContext, OpMeta, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;
use crate::tone::shared::{HL_RECONSTRUCT_BIAS, HL_RECONSTRUCT_KNEE, RAW_SENSOR_WHITE};
use std::sync::LazyLock;

pub struct WhiteBalanceOp;

static WB_WGSL: LazyLock<String> = LazyLock::new(|| {
    format!(
        concat!(
            "fn white_balance_apply(c: vec3<f32>, w: vec4<f32>) -> vec3<f32> {{\n",
            "  var wb_lin = vec3<f32>(c.r * w.r, c.g * w.g, c.b * w.b);\n",
            "  if (w.w < 0.5) {{ return wb_lin; }}\n",
            "  let cr = smoothstep({knee:?}, {white:?}, c.r);\n",
            "  let cg = smoothstep({knee:?}, {white:?}, c.g);\n",
            "  let cb = smoothstep({knee:?}, {white:?}, c.b);\n",
            "  if (max(cr, max(cg, cb)) <= 0.0) {{ return wb_lin; }}\n",
            "  let ur = 1.0 - cr;\n",
            "  let ug = 1.0 - cg;\n",
            "  let ub = 1.0 - cb;\n",
            "  let wmax = max(wb_lin.r, max(wb_lin.g, wb_lin.b));\n",
            "  let recon_target = (ur * wb_lin.r + ug * wb_lin.g + ub * wb_lin.b + {bias:?} * wmax) / (ur + ug + ub + {bias:?});\n",
            "  if (wb_lin.r < recon_target) {{ wb_lin.r = wb_lin.r + (recon_target - wb_lin.r) * cr; }}\n",
            "  if (wb_lin.g < recon_target) {{ wb_lin.g = wb_lin.g + (recon_target - wb_lin.g) * cg; }}\n",
            "  if (wb_lin.b < recon_target) {{ wb_lin.b = wb_lin.b + (recon_target - wb_lin.b) * cb; }}\n",
            "  return wb_lin;\n",
            "}}",
        ),
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
