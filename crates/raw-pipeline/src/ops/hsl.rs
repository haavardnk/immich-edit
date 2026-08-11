use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::{Edits, HSL_BANDS};

pub struct HslOp;

impl Op for HslOp {
    fn id(&self) -> &'static str {
        "hsl"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
    }
    fn stage(&self) -> Stage {
        Stage::Color
    }
    fn order(&self) -> i32 {
        -10
    }
    fn is_active(&self, edits: &Edits) -> bool {
        !edits.color.hsl.is_zero()
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if !self.is_active(edits) {
            return None;
        }
        let arr: Vec<serde_json::Value> = edits
            .color
            .hsl
            .bands
            .iter()
            .map(|b| serde_json::json!({"hue": b.hue, "sat": b.sat, "lum": b.lum}))
            .collect();
        Some(serde_json::json!({ "bands": arr }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let Some(arr) = value.get("bands").and_then(|v| v.as_array()) else {
            return;
        };
        for (i, item) in arr.iter().take(HSL_BANDS).enumerate() {
            let band = &mut edits.color.hsl.bands[i];
            if let Some(v) = item.get("hue").and_then(|v| v.as_f64()) {
                band.hue = v;
            }
            if let Some(v) = item.get("sat").and_then(|v| v.as_f64()) {
                band.sat = v;
            }
            if let Some(v) = item.get("lum").and_then(|v| v.as_f64()) {
                band.lum = v;
            }
        }
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "hsl",
            functions: include_str!("../../assets/shaders/ops/hsl.wgsl"),
            apply: "lin = hsl_apply(lin);",
            vec4_count: HSL_BANDS,
        })
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        if !self.is_active(edits) {
            return None;
        }
        let bands = edits.color.hsl.bands;
        let hue_shifts: [f32; HSL_BANDS] =
            std::array::from_fn(|i| (bands[i].hue as f32) / 100.0 * 30.0);
        let sat_gains: [f32; HSL_BANDS] = std::array::from_fn(|i| (bands[i].sat as f32) / 100.0);
        let lum_gains: [f32; HSL_BANDS] = std::array::from_fn(|i| (bands[i].lum as f32) / 100.0);
        Some(CpuFusedOp::Hsl {
            hue_shifts,
            sat_gains,
            lum_gains,
        })
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        for (i, band) in edits.color.hsl.bands.iter().enumerate() {
            dst[i * 4] = band.hue as f32;
            dst[i * 4 + 1] = band.sat as f32;
            dst[i * 4 + 2] = band.lum as f32;
            dst[i * 4 + 3] = 0.0;
        }
    }
}
