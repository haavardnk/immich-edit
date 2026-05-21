use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::Edits;

pub struct HighlightsShadowsOp;

impl EditOperator for HighlightsShadowsOp {
    fn id(&self) -> &'static str {
        "highlights_shadows"
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        10
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.highlights != 0.0 || edits.shadows != 0.0
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let hl = edits.highlights as f32 / 100.0;
        let sh = edits.shadows as f32 / 100.0;
        for v in image.rgb.iter_mut() {
            let x = v.clamp(0.0, 2.0);
            if x > 0.5 {
                *v = x + hl * (1.0 - x) * 0.5;
            } else {
                *v = x + sh * x * 0.5;
            }
        }
        Ok(())
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "highlights_shadows",
            functions: "fn hl_sh_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> { let hl = p.x; let sh = p.y; if (hl == 0.0 && sh == 0.0) { return c; } var out_v = vec3<f32>(0.0); for (var i = 0u; i < 3u; i = i + 1u) { let x = clamp(c[i], 0.0, 2.0); if (x > 0.5) { out_v[i] = x + hl * (1.0 - x) * 0.5; } else { out_v[i] = x + sh * x * 0.5; } } return out_v; }",
            apply: "lin = hl_sh_apply(lin, p.highlights_shadows);",
        })
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32; 4]) {
        dst[0] = edits.highlights as f32 / 100.0;
        dst[1] = edits.shadows as f32 / 100.0;
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.highlights == 0.0 && edits.shadows == 0.0 {
            return None;
        }
        Some(serde_json::json!({
            "highlights": edits.highlights,
            "shadows": edits.shadows,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("highlights").and_then(|v| v.as_f64()) {
            edits.highlights = v;
        }
        if let Some(v) = value.get("shadows").and_then(|v| v.as_f64()) {
            edits.shadows = v;
        }
    }
}
