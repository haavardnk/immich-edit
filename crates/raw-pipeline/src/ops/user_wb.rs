use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::Edits;
use rayon::prelude::*;

pub struct UserWbOp;

impl EditOperator for UserWbOp {
    fn id(&self) -> &'static str {
        "white_balance"
    }
    fn stage(&self) -> Stage {
        Stage::WhiteBalance
    }
    fn order(&self) -> i32 {
        20
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.basic.wb_temp != 0.0 || edits.basic.wb_tint != 0.0
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let m = crate::color::user_wb_matrix(edits.basic.wb_temp, edits.basic.wb_tint);
        image.rgb.par_chunks_exact_mut(3).for_each(|px| {
            let r = px[0];
            let g = px[1];
            let b = px[2];
            px[0] = m[0][0] * r + m[0][1] * g + m[0][2] * b;
            px[1] = m[1][0] * r + m[1][1] * g + m[1][2] * b;
            px[2] = m[2][0] * r + m[2][1] * g + m[2][2] * b;
        });
        Ok(())
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
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        let m = crate::color::user_wb_matrix(edits.basic.wb_temp, edits.basic.wb_tint);
        for (row_idx, row) in m.iter().enumerate() {
            let off = row_idx * 4;
            dst[off] = row[0];
            dst[off + 1] = row[1];
            dst[off + 2] = row[2];
            dst[off + 3] = 0.0;
        }
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.basic.wb_temp == 0.0 && edits.basic.wb_tint == 0.0 {
            return None;
        }
        Some(serde_json::json!({
            "temp": edits.basic.wb_temp,
            "tint": edits.basic.wb_tint,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("temp").and_then(|v| v.as_f64()) {
            edits.basic.wb_temp = v;
        }
        if let Some(v) = value.get("tint").and_then(|v| v.as_f64()) {
            edits.basic.wb_tint = v;
        }
    }
}
