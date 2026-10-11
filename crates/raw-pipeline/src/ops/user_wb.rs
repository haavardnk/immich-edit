use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;

pub const USER_WB_OP_ID: &str = "white_balance";

pub struct UserWbOp;

impl Op for UserWbOp {
    fn id(&self) -> &'static str {
        USER_WB_OP_ID
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        -10
    }
    fn is_active(&self, edits: &Edits) -> bool {
        !edits.basic.white_balance.is_as_shot()
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.basic.white_balance.is_as_shot() {
            return None;
        }
        serde_json::to_value(edits.basic.white_balance).ok()
    }
    fn apply_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Ok(wb) = serde_json::from_value(value.clone()) {
            edits.basic.white_balance = wb;
        }
    }
    fn cpu_fused(&self, edits: &Edits, ctx: &OpContext) -> Option<CpuFusedOp> {
        let m = ctx.render.white.user_matrix(&edits.basic.white_balance);
        Some(CpuFusedOp::ColorMatrix { m })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "user_wb",
            functions: concat!(
                "fn user_wb_apply(c: vec3<f32>) -> vec3<f32> {\n",
                "  let m0 = p.user_wb[0];\n",
                "  let m1 = p.user_wb[1];\n",
                "  let m2 = p.user_wb[2];\n",
                "  return vec3<f32>(\n",
                "    m0.x * c.r + m0.y * c.g + m0.z * c.b,\n",
                "    m1.x * c.r + m1.y * c.g + m1.z * c.b,\n",
                "    m2.x * c.r + m2.y * c.g + m2.z * c.b\n",
                "  );\n",
                "}\n",
            ),
            apply: "lin = user_wb_apply(lin);",
            vec4_count: 3,
        })
    }
    fn write_gpu_uniform(&self, edits: &Edits, ctx: &OpContext, dst: &mut [f32]) {
        let m = ctx.render.white.user_matrix(&edits.basic.white_balance);
        for (row_idx, row) in m.iter().enumerate() {
            let off = row_idx * 4;
            dst[off] = row[0];
            dst[off + 1] = row[1];
            dst[off + 2] = row[2];
            dst[off + 3] = 0.0;
        }
    }
}
