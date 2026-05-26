use super::{FusedOp, GpuOp, OpContext, OpMeta, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::{CURVE_LUT_SIZE, CurvePoint, CurvePoints, CurvesEdits, Edits};

pub struct CurvesOp;

#[derive(Clone, Debug)]
pub struct CurveLuts {
    pub composite: [f32; CURVE_LUT_SIZE],
    pub r: [f32; CURVE_LUT_SIZE],
    pub g: [f32; CURVE_LUT_SIZE],
    pub b: [f32; CURVE_LUT_SIZE],
    pub luma: [f32; CURVE_LUT_SIZE],
}

impl CurveLuts {
    pub fn from_edits(curves: &CurvesEdits) -> Self {
        Self {
            composite: build_lut(&curves.composite.points),
            r: build_lut(&curves.r.points),
            g: build_lut(&curves.g.points),
            b: build_lut(&curves.b.points),
            luma: build_lut(&curves.luma.points),
        }
    }
}

pub fn build_lut(points: &[CurvePoint]) -> [f32; CURVE_LUT_SIZE] {
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

#[inline(always)]
pub fn sample_lut(lut: &[f32; CURVE_LUT_SIZE], v: f32) -> f32 {
    let x = v.clamp(0.0, 1.0) * (CURVE_LUT_SIZE - 1) as f32;
    let idx = (x as usize).min(CURVE_LUT_SIZE - 2);
    let frac = x - idx as f32;
    lut[idx] * (1.0 - frac) + lut[idx + 1] * frac
}

#[inline(always)]
pub fn apply_curves_pixel(luts: &CurveLuts, r: &mut f32, g: &mut f32, b: &mut f32) {
    *r = sample_lut(&luts.composite, *r);
    *g = sample_lut(&luts.composite, *g);
    *b = sample_lut(&luts.composite, *b);
    *r = sample_lut(&luts.r, *r);
    *g = sample_lut(&luts.g, *g);
    *b = sample_lut(&luts.b, *b);
    let y0 = 0.2126 * *r + 0.7152 * *g + 0.0722 * *b;
    let y0c = y0.clamp(0.0, 1.0);
    let y1 = sample_lut(&luts.luma, y0c);
    if y0 < 1e-5 {
        *r = y1;
        *g = y1;
        *b = y1;
    } else {
        let scale = y1 / y0;
        *r *= scale;
        *g *= scale;
        *b *= scale;
    }
}

impl OpMeta for CurvesOp {
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
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let c = &edits.basic.curves;
        if c.is_identity() {
            return None;
        }
        let mut obj = serde_json::Map::new();
        let put =
            |obj: &mut serde_json::Map<String, serde_json::Value>, key: &str, pts: &CurvePoints| {
                if !pts.is_identity() {
                    let arr: Vec<serde_json::Value> = pts
                        .points
                        .iter()
                        .map(|p| serde_json::json!([p.x, p.y]))
                        .collect();
                    obj.insert(key.into(), serde_json::Value::Array(arr));
                }
            };
        put(&mut obj, "composite", &c.composite);
        put(&mut obj, "r", &c.r);
        put(&mut obj, "g", &c.g);
        put(&mut obj, "b", &c.b);
        put(&mut obj, "luma", &c.luma);
        Some(serde_json::Value::Object(obj))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(arr) = value.get("points").and_then(|v| v.as_array()) {
            if let Some(pts) = decode_points(arr) {
                edits.basic.curves.composite = CurvePoints { points: pts };
            }
            return;
        }
        let read = |key: &str, dst: &mut CurvePoints| {
            if let Some(arr) = value.get(key).and_then(|v| v.as_array()) {
                if let Some(pts) = decode_points(arr) {
                    *dst = CurvePoints { points: pts };
                }
            }
        };
        read("composite", &mut edits.basic.curves.composite);
        read("r", &mut edits.basic.curves.r);
        read("g", &mut edits.basic.curves.g);
        read("b", &mut edits.basic.curves.b);
        read("luma", &mut edits.basic.curves.luma);
    }
}

impl FusedOp for CurvesOp {
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        Some(CpuFusedOp::Curves {
            luts: Box::new(CurveLuts::from_edits(&edits.basic.curves)),
        })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "curves",
            functions: concat!(
                "fn curves_get(base: u32, idx: u32) -> f32 {\n",
                "  let abs_idx = base + idx;\n",
                "  let vi = abs_idx / 4u;\n",
                "  let ci = abs_idx % 4u;\n",
                "  return p.curves[vi][ci];\n",
                "}\n",
                "fn curves_sample(base: u32, x: f32) -> f32 {\n",
                "  let cx = clamp(x, 0.0, 1.0) * 15.0;\n",
                "  let idx = u32(cx);\n",
                "  let frac = cx - f32(idx);\n",
                "  let v0 = curves_get(base, idx);\n",
                "  let v1 = curves_get(base, min(idx + 1u, 15u));\n",
                "  return mix(v0, v1, frac);\n",
                "}\n",
                "fn curves_apply(c: vec3<f32>) -> vec3<f32> {\n",
                "  var r = curves_sample(0u, c.x);\n",
                "  var g = curves_sample(0u, c.y);\n",
                "  var b = curves_sample(0u, c.z);\n",
                "  r = curves_sample(16u, r);\n",
                "  g = curves_sample(32u, g);\n",
                "  b = curves_sample(48u, b);\n",
                "  let y0 = 0.2126 * r + 0.7152 * g + 0.0722 * b;\n",
                "  let y0c = clamp(y0, 0.0, 1.0);\n",
                "  let y1 = curves_sample(64u, y0c);\n",
                "  if (y0 < 1e-5) {\n",
                "    return vec3<f32>(y1);\n",
                "  }\n",
                "  let scale = y1 / y0;\n",
                "  return vec3<f32>(r * scale, g * scale, b * scale);\n",
                "}\n",
            ),
            apply: "lin = curves_apply(lin);",
            vec4_count: 20,
            kind: crate::ops::GpuOpKind::Normal,
        })
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        let luts = CurveLuts::from_edits(&edits.basic.curves);
        let blocks: [&[f32; CURVE_LUT_SIZE]; 5] =
            [&luts.composite, &luts.r, &luts.g, &luts.b, &luts.luma];
        for (i, block) in blocks.iter().enumerate() {
            let base = i * CURVE_LUT_SIZE;
            dst[base..base + CURVE_LUT_SIZE].copy_from_slice(*block);
        }
    }
}

fn decode_points(arr: &[serde_json::Value]) -> Option<Vec<CurvePoint>> {
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
    if pts.len() >= 2 { Some(pts) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(x: f64, y: f64) -> CurvePoint {
        CurvePoint { x, y }
    }

    fn s_curve_pts() -> Vec<CurvePoint> {
        vec![
            pt(0.0, 0.0),
            pt(0.25, 0.15),
            pt(0.5, 0.5),
            pt(0.75, 0.85),
            pt(1.0, 1.0),
        ]
    }

    #[test]
    fn identity_lut_is_linear() {
        let pts = vec![pt(0.0, 0.0), pt(1.0, 1.0)];
        let lut = build_lut(&pts);
        for (i, v) in lut.iter().enumerate() {
            let expected = i as f32 / (CURVE_LUT_SIZE - 1) as f32;
            if (v - expected).abs() >= 0.01 {
                panic!("lut[{i}] = {v}, expected {expected}");
            }
        }
    }

    #[test]
    fn s_curve_midpoint() {
        let lut = build_lut(&s_curve_pts());
        let mid = lut[CURVE_LUT_SIZE / 2];
        if (mid - 0.5).abs() >= 0.05 {
            panic!("midpoint was {mid}");
        }
    }

    fn luts_with(curves: CurvesEdits) -> CurveLuts {
        CurveLuts::from_edits(&curves)
    }

    #[test]
    fn composite_curve_lifts_all_channels() {
        let c = CurvesEdits {
            composite: CurvePoints {
                points: vec![pt(0.0, 0.0), pt(0.5, 0.7), pt(1.0, 1.0)],
            },
            ..Default::default()
        };
        let luts = luts_with(c);
        let (mut r, mut g, mut b) = (0.5_f32, 0.5_f32, 0.5_f32);
        apply_curves_pixel(&luts, &mut r, &mut g, &mut b);
        if r <= 0.55 || (r - g).abs() > 1e-4 || (g - b).abs() > 1e-4 {
            panic!("composite did not lift uniformly: {r} {g} {b}");
        }
    }

    #[test]
    fn red_curve_only_shifts_red() {
        let c = CurvesEdits {
            r: CurvePoints {
                points: vec![pt(0.0, 0.0), pt(0.5, 0.8), pt(1.0, 1.0)],
            },
            ..Default::default()
        };
        let luts = luts_with(c);
        let (mut r, mut g, mut b) = (0.5_f32, 0.5_f32, 0.5_f32);
        apply_curves_pixel(&luts, &mut r, &mut g, &mut b);
        let dr = (r - 0.5).abs();
        let dg = (g - 0.5).abs();
        let db = (b - 0.5).abs();
        if dr < 0.05 || dg > 0.05 || db > 0.05 {
            panic!("red curve leaked: r={r} g={g} b={b}");
        }
    }

    #[test]
    fn luma_curve_on_grayscale_acts_like_composite() {
        let c = CurvesEdits {
            luma: CurvePoints {
                points: vec![pt(0.0, 0.0), pt(0.5, 0.7), pt(1.0, 1.0)],
            },
            ..Default::default()
        };
        let luts = luts_with(c);
        let (mut r, mut g, mut b) = (0.5_f32, 0.5_f32, 0.5_f32);
        apply_curves_pixel(&luts, &mut r, &mut g, &mut b);
        if (r - g).abs() > 1e-4 || (g - b).abs() > 1e-4 || r <= 0.55 {
            panic!("luma on gray failed: {r} {g} {b}");
        }
    }

    #[test]
    fn luma_curve_preserves_hue_ratio() {
        let c = CurvesEdits {
            luma: CurvePoints {
                points: vec![pt(0.0, 0.0), pt(0.4, 0.55), pt(1.0, 1.0)],
            },
            ..Default::default()
        };
        let luts = luts_with(c);
        let (r0, g0, b0) = (0.8_f32, 0.2_f32, 0.1_f32);
        let (mut r, mut g, mut b) = (r0, g0, b0);
        apply_curves_pixel(&luts, &mut r, &mut g, &mut b);
        let s = r / r0;
        if (g / g0 - s).abs() > 1e-3 || (b / b0 - s).abs() > 1e-3 {
            panic!(
                "hue ratio drifted: r/g/b scales {s}, {}, {}",
                g / g0,
                b / b0
            );
        }
    }

    #[test]
    fn identity_curves_is_noop() {
        let luts = luts_with(CurvesEdits::default());
        let (mut r, mut g, mut b) = (0.42_f32, 0.18_f32, 0.71_f32);
        apply_curves_pixel(&luts, &mut r, &mut g, &mut b);
        if (r - 0.42).abs() > 1e-3 || (g - 0.18).abs() > 1e-3 || (b - 0.71).abs() > 1e-3 {
            panic!("identity curves changed pixel: {r} {g} {b}");
        }
    }

    #[test]
    fn manifest_legacy_points_decodes_into_composite() {
        let mut edits = Edits::default();
        let v = serde_json::json!({ "points": [[0.0, 0.1], [1.0, 0.9]] });
        CurvesOp.from_doc(&v, &mut edits);
        if edits.basic.curves.composite.is_identity() {
            panic!("legacy points did not populate composite");
        }
        if !edits.basic.curves.r.is_identity()
            || !edits.basic.curves.g.is_identity()
            || !edits.basic.curves.b.is_identity()
            || !edits.basic.curves.luma.is_identity()
        {
            panic!("legacy points leaked into per-channel curves");
        }
    }

    #[test]
    fn manifest_sparse_structured_roundtrip() {
        let mut edits = Edits::default();
        edits.basic.curves = CurvesEdits {
            r: CurvePoints {
                points: vec![pt(0.0, 0.0), pt(0.5, 0.7), pt(1.0, 1.0)],
            },
            luma: CurvePoints {
                points: vec![pt(0.0, 0.05), pt(1.0, 1.0)],
            },
            ..Default::default()
        };
        let doc = CurvesOp.to_doc(&edits).expect("doc");
        let obj = doc.as_object().expect("object");
        if obj.contains_key("composite") || obj.contains_key("g") || obj.contains_key("b") {
            panic!("identity sub-curves were not omitted: {obj:?}");
        }
        if !obj.contains_key("r") || !obj.contains_key("luma") {
            panic!("missing r/luma in doc: {obj:?}");
        }
        let mut back = Edits::default();
        CurvesOp.from_doc(&doc, &mut back);
        if back.basic.curves != edits.basic.curves {
            panic!(
                "roundtrip mismatch: {:?} vs {:?}",
                back.basic.curves, edits.basic.curves
            );
        }
    }
}
