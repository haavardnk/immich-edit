use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::Edits;

pub struct WhiteBalanceOp;

fn compute_wb(raw: [f32; 4], temp: f64, tint: f64) -> [f32; 4] {
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
    let t = temp as f32 / 100.0;
    let ti = tint as f32 / 100.0;
    c[0] *= 1.0 + t * 0.5;
    c[2] *= 1.0 - t * 0.5;
    c[1] *= 1.0 - ti * 0.3;
    c
}

impl EditOperator for WhiteBalanceOp {
    fn id(&self) -> &'static str {
        "white_balance"
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
        edits: &Edits,
    ) -> PipelineResult<()> {
        let coeffs = compute_wb(ctx.wb_coeffs, edits.wb_temp, edits.wb_tint);
        let pixels = image.pixel_count();
        for i in 0..pixels {
            let idx = i * 3;
            image.rgb[idx] *= coeffs[0];
            image.rgb[idx + 1] *= coeffs[1];
            image.rgb[idx + 2] *= coeffs[2];
        }
        Ok(())
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "white_balance",
            functions: "fn white_balance_apply(c: vec3<f32>, w: vec4<f32>) -> vec3<f32> { return vec3<f32>(c.r * w.r, c.g * w.g, c.b * w.b); }",
            apply: "lin = white_balance_apply(lin, p.white_balance);",
        })
    }
    fn write_gpu_uniform(&self, edits: &Edits, ctx: &OpContext, dst: &mut [f32; 4]) {
        let c = compute_wb(ctx.wb_coeffs, edits.wb_temp, edits.wb_tint);
        dst[0] = c[0];
        dst[1] = c[1];
        dst[2] = c[2];
        dst[3] = 1.0;
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.wb_temp == 0.0 && edits.wb_tint == 0.0 {
            return None;
        }
        Some(serde_json::json!({
            "temp": edits.wb_temp,
            "tint": edits.wb_tint,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("temp").and_then(|v| v.as_f64()) {
            edits.wb_temp = v;
        }
        if let Some(v) = value.get("tint").and_then(|v| v.as_f64()) {
            edits.wb_tint = v;
        }
    }
}
