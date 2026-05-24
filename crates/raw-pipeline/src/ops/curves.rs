use super::{EditOperator, GpuOp, LinearImage, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::fused::CpuFusedOp;
use crate::edits::{CURVE_LUT_SIZE, CurvePoint, CurvePoints, Edits};
use rayon::prelude::*;

pub struct CurvesOp;

fn build_lut(points: &[CurvePoint]) -> [f32; CURVE_LUT_SIZE] {
    let mut lut = [0.0f32; CURVE_LUT_SIZE];
    if points.len() < 2 {
        for (i, v) in lut.iter_mut().enumerate() {
            *v = i as f32 / (CURVE_LUT_SIZE - 1) as f32;
        }
        return lut;
    }
    for (i, v) in lut.iter_mut().enumerate() {
        let x = i as f64 / (CURVE_LUT_SIZE - 1) as f64;
        *v = interpolate_monotone(points, x) as f32;
    }
    lut
}

fn interpolate_monotone(points: &[CurvePoint], x: f64) -> f64 {
    if x <= points[0].x {
        return points[0].y;
    }
    if x >= points[points.len() - 1].x {
        return points[points.len() - 1].y;
    }
    let mut idx = 0;
    for i in 0..points.len() - 1 {
        if x >= points[i].x && x <= points[i + 1].x {
            idx = i;
            break;
        }
    }
    let (x0, y0) = (points[idx].x, points[idx].y);
    let (x1, y1) = (points[idx + 1].x, points[idx + 1].y);
    let dx = x1 - x0;
    if dx < 1e-10 {
        return y0;
    }
    let t = (x - x0) / dx;
    let m0 = tangent_at(points, idx);
    let m1 = tangent_at(points, idx + 1);
    let t2 = t * t;
    let t3 = t2 * t;
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    (h00 * y0 + h10 * dx * m0 + h01 * y1 + h11 * dx * m1).clamp(0.0, 1.0)
}

fn tangent_at(points: &[CurvePoint], i: usize) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }
    if i == 0 {
        return (points[1].y - points[0].y) / (points[1].x - points[0].x).max(1e-10);
    }
    if i == points.len() - 1 {
        let n = points.len();
        return (points[n - 1].y - points[n - 2].y)
            / (points[n - 1].x - points[n - 2].x).max(1e-10);
    }
    let d0 = (points[i].y - points[i - 1].y) / (points[i].x - points[i - 1].x).max(1e-10);
    let d1 = (points[i + 1].y - points[i].y) / (points[i + 1].x - points[i].x).max(1e-10);
    if d0.signum() != d1.signum() {
        0.0
    } else {
        (d0 + d1) * 0.5
    }
}

fn apply_lut(lut: &[f32; CURVE_LUT_SIZE], v: f32) -> f32 {
    let x = v.clamp(0.0, 1.0) * (CURVE_LUT_SIZE - 1) as f32;
    let idx = (x as usize).min(CURVE_LUT_SIZE - 2);
    let frac = x - idx as f32;
    lut[idx] * (1.0 - frac) + lut[idx + 1] * frac
}

impl EditOperator for CurvesOp {
    fn id(&self) -> &'static str {
        "curves"
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        25
    }
    fn is_active(&self, edits: &Edits) -> bool {
        !edits.basic.curves.is_identity()
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let lut = build_lut(&edits.basic.curves.points);
        image.rgb.par_chunks_mut(3).for_each(|px| {
            px[0] = apply_lut(&lut, px[0]);
            px[1] = apply_lut(&lut, px[1]);
            px[2] = apply_lut(&lut, px[2]);
        });
        Ok(())
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        let lut = build_lut(&edits.basic.curves.points);
        Some(CpuFusedOp::Curves { lut })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "curves",
            functions: concat!(
                "fn curves_get(idx: u32) -> f32 {\n",
                "  let vi = idx / 4u;\n",
                "  let ci = idx % 4u;\n",
                "  if (vi == 0u) { return p.curves[0][ci]; }\n",
                "  if (vi == 1u) { return p.curves[1][ci]; }\n",
                "  if (vi == 2u) { return p.curves[2][ci]; }\n",
                "  return p.curves[3][ci];\n",
                "}\n",
                "fn curves_sample(x: f32) -> f32 {\n",
                "  let cx = clamp(x, 0.0, 1.0) * 15.0;\n",
                "  let idx = u32(cx);\n",
                "  let frac = cx - f32(idx);\n",
                "  let v0 = curves_get(idx);\n",
                "  let v1 = curves_get(min(idx + 1u, 15u));\n",
                "  return mix(v0, v1, frac);\n",
                "}\n",
                "fn curves_apply(c: vec3<f32>) -> vec3<f32> {\n",
                "  return vec3<f32>(curves_sample(c.x), curves_sample(c.y), curves_sample(c.z));\n",
                "}\n",
            ),
            apply: "lin = curves_apply(lin);",
            vec4_count: 4,
            kind: crate::ops::GpuOpKind::Normal,
        })
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        let lut = build_lut(&edits.basic.curves.points);
        dst[..CURVE_LUT_SIZE].copy_from_slice(&lut[..CURVE_LUT_SIZE]);
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.basic.curves.is_identity() {
            return None;
        }
        Some(serde_json::json!({
            "points": edits.basic.curves.points.iter()
                .map(|p| serde_json::json!([p.x, p.y]))
                .collect::<Vec<_>>()
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(arr) = value.get("points").and_then(|v| v.as_array()) {
            let pts: Vec<CurvePoint> = arr
                .iter()
                .filter_map(|p| {
                    let a = p.as_array()?;
                    Some(CurvePoint {
                        x: a.first()?.as_f64()?,
                        y: a.get(1)?.as_f64()?,
                    })
                })
                .collect();
            if pts.len() >= 2 {
                edits.basic.curves = CurvePoints { points: pts };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(x: f64, y: f64) -> CurvePoint {
        CurvePoint { x, y }
    }

    #[test]
    fn identity_lut_is_linear() {
        let pts = vec![pt(0.0, 0.0), pt(1.0, 1.0)];
        let lut = build_lut(&pts);
        for (i, v) in lut.iter().enumerate() {
            let expected = i as f32 / (CURVE_LUT_SIZE - 1) as f32;
            assert!(
                (v - expected).abs() < 0.01,
                "lut[{i}] = {v}, expected {expected}"
            );
        }
    }

    #[test]
    fn s_curve_midpoint() {
        let pts = vec![
            pt(0.0, 0.0),
            pt(0.25, 0.15),
            pt(0.5, 0.5),
            pt(0.75, 0.85),
            pt(1.0, 1.0),
        ];
        let lut = build_lut(&pts);
        let mid = lut[CURVE_LUT_SIZE / 2];
        assert!((mid - 0.5).abs() < 0.05, "midpoint was {mid}");
    }
}
