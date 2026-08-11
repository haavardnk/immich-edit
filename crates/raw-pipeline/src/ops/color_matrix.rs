use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;

pub struct ColorMatrixOp;

impl Op for ColorMatrixOp {
    fn id(&self) -> &'static str {
        "color_matrix"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
    }
    fn stage(&self) -> Stage {
        Stage::WhiteBalance
    }
    fn order(&self) -> i32 {
        10
    }
    fn is_active(&self, _edits: &Edits) -> bool {
        true
    }
    fn cpu_fused(&self, _edits: &Edits, ctx: &OpContext) -> Option<CpuFusedOp> {
        if !ctx.render.is_raw {
            return None;
        }
        Some(CpuFusedOp::ColorMatrix {
            m: ctx.render.cam_to_srgb,
        })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "color_matrix",
            functions: concat!(
                "fn color_matrix_apply(c: vec3<f32>) -> vec3<f32> {\n",
                "  let m0 = p.color_matrix[0];\n",
                "  let m1 = p.color_matrix[1];\n",
                "  let m2 = p.color_matrix[2];\n",
                "  if (m0.w < 0.5) { return c; }\n",
                "  return vec3<f32>(\n",
                "    m0.x * c.r + m0.y * c.g + m0.z * c.b,\n",
                "    m1.x * c.r + m1.y * c.g + m1.z * c.b,\n",
                "    m2.x * c.r + m2.y * c.g + m2.z * c.b\n",
                "  );\n",
                "}\n",
            ),
            apply: "lin = color_matrix_apply(lin);",
            vec4_count: 3,
        })
    }
    fn write_gpu_uniform(&self, _edits: &Edits, ctx: &OpContext, dst: &mut [f32]) {
        let m = ctx.render.cam_to_srgb;
        for (row_idx, row) in m.iter().enumerate() {
            let off = row_idx * 4;
            dst[off] = row[0];
            dst[off + 1] = row[1];
            dst[off + 2] = row[2];
            dst[off + 3] = 0.0;
        }
        dst[3] = if ctx.render.is_raw { 1.0 } else { 0.0 };
    }
}
