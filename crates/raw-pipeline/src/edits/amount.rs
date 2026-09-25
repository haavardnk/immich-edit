use super::{
    BasicEdits, ColorEdits, ColorGradeEdits, ColorGradeRegion, CurvePoint, CurvePoints,
    CurvesEdits, DetailEdits, Edits, EffectsEdits, HslBand, HslEdits, Lut3dEdits, ToneEdits,
};

pub const LOOK_AMOUNT_FULL: f64 = 100.0;
const LOOK_AMOUNT_MAX: f64 = 200.0;

fn lerp(neutral: f64, preset: f64, t: f64, lo: f64, hi: f64) -> f64 {
    (neutral + (preset - neutral) * t).clamp(lo, hi)
}

fn signed(neutral: f64, preset: f64, t: f64) -> f64 {
    lerp(neutral, preset, t, -100.0, 100.0)
}

fn unsigned(neutral: f64, preset: f64, t: f64) -> f64 {
    lerp(neutral, preset, t, 0.0, 100.0)
}

fn scale_curve(curve: &CurvePoints, t: f64) -> CurvePoints {
    CurvePoints {
        points: curve
            .points
            .iter()
            .map(|p| CurvePoint {
                x: p.x,
                y: (p.x + (p.y - p.x) * t).clamp(0.0, 1.0),
            })
            .collect(),
    }
}

fn scale_basic(p: &BasicEdits, n: &BasicEdits, t: f64) -> BasicEdits {
    BasicEdits {
        exposure_ev: lerp(n.exposure_ev, p.exposure_ev, t, -5.0, 5.0),
        brightness: signed(n.brightness, p.brightness, t),
        contrast: signed(n.contrast, p.contrast, t),
        saturation: signed(n.saturation, p.saturation, t),
        vibrance: signed(n.vibrance, p.vibrance, t),
        wb_temp: signed(n.wb_temp, p.wb_temp, t),
        wb_tint: signed(n.wb_tint, p.wb_tint, t),
        texture: signed(n.texture, p.texture, t),
        clarity: signed(n.clarity, p.clarity, t),
        dehaze: signed(n.dehaze, p.dehaze, t),
        curves: CurvesEdits {
            composite: scale_curve(&p.curves.composite, t),
            r: scale_curve(&p.curves.r, t),
            g: scale_curve(&p.curves.g, t),
            b: scale_curve(&p.curves.b, t),
            luma: scale_curve(&p.curves.luma, t),
        },
    }
}

fn scale_tone(p: &ToneEdits, n: &ToneEdits, t: f64) -> ToneEdits {
    ToneEdits {
        highlights: signed(n.highlights, p.highlights, t),
        shadows: signed(n.shadows, p.shadows, t),
        blacks: signed(n.blacks, p.blacks, t),
        whites: signed(n.whites, p.whites, t),
    }
}

fn scale_region(p: &ColorGradeRegion, n: &ColorGradeRegion, t: f64) -> ColorGradeRegion {
    ColorGradeRegion {
        hue: p.hue,
        sat: unsigned(n.sat, p.sat, t),
        lum: lerp(n.lum, p.lum, t, -50.0, 50.0),
    }
}

fn scale_color(p: &ColorEdits, n: &ColorEdits, t: f64) -> ColorEdits {
    let grade = &p.color_grade;
    let neutral_grade = &n.color_grade;
    let has_lut = p.lut_3d.lut_id.as_deref().is_some_and(|id| !id.is_empty());
    ColorEdits {
        hsl: HslEdits {
            bands: p.hsl.bands.map(|band| HslBand {
                hue: signed(0.0, band.hue, t),
                sat: signed(0.0, band.sat, t),
                lum: signed(0.0, band.lum, t),
            }),
        },
        color_grade: ColorGradeEdits {
            shadows: scale_region(&grade.shadows, &neutral_grade.shadows, t),
            midtones: scale_region(&grade.midtones, &neutral_grade.midtones, t),
            highlights: scale_region(&grade.highlights, &neutral_grade.highlights, t),
            global: scale_region(&grade.global, &neutral_grade.global, t),
            balance: signed(neutral_grade.balance, grade.balance, t),
            blend: unsigned(neutral_grade.blend, grade.blend, t),
        },
        lut_3d: Lut3dEdits {
            lut_id: p.lut_3d.lut_id.clone(),
            amount: if has_lut {
                unsigned(0.0, p.lut_3d.amount, t)
            } else {
                p.lut_3d.amount
            },
        },
        dcp: p.dcp.clone(),
    }
}

fn scale_detail(p: &DetailEdits, n: &DetailEdits, t: f64) -> DetailEdits {
    DetailEdits {
        luma_nr_amount: unsigned(n.luma_nr_amount, p.luma_nr_amount, t),
        luma_nr_detail: unsigned(n.luma_nr_detail, p.luma_nr_detail, t),
        luma_nr_contrast: unsigned(n.luma_nr_contrast, p.luma_nr_contrast, t),
        color_nr_amount: unsigned(n.color_nr_amount, p.color_nr_amount, t),
        color_nr_detail: unsigned(n.color_nr_detail, p.color_nr_detail, t),
        color_nr_smoothness: unsigned(n.color_nr_smoothness, p.color_nr_smoothness, t),
        ..*p
    }
}

fn scale_effects(p: &EffectsEdits, n: &EffectsEdits, t: f64) -> EffectsEdits {
    EffectsEdits {
        vignette_amount: signed(n.vignette_amount, p.vignette_amount, t),
        vignette_midpoint: unsigned(n.vignette_midpoint, p.vignette_midpoint, t),
        vignette_feather: unsigned(n.vignette_feather, p.vignette_feather, t),
        vignette_roundness: signed(n.vignette_roundness, p.vignette_roundness, t),
        grain_amount: unsigned(n.grain_amount, p.grain_amount, t),
        grain_size: unsigned(n.grain_size, p.grain_size, t),
        grain_roughness: unsigned(n.grain_roughness, p.grain_roughness, t),
    }
}

impl Edits {
    pub fn with_look_amount(&self, amount: f64) -> Self {
        let t = amount.clamp(0.0, LOOK_AMOUNT_MAX) / LOOK_AMOUNT_FULL;
        if t == 1.0 {
            return self.clone();
        }
        let n = Self::default();
        if t == 0.0 {
            return Self {
                basic: n.basic,
                tone: n.tone,
                color: n.color,
                detail: n.detail,
                effects: n.effects,
                lens: n.lens,
                ..self.clone()
            };
        }
        Self {
            basic: scale_basic(&self.basic, &n.basic, t),
            tone: scale_tone(&self.tone, &n.tone, t),
            color: scale_color(&self.color, &n.color, t),
            detail: scale_detail(&self.detail, &n.detail, t),
            effects: scale_effects(&self.effects, &n.effects, t),
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    const CASES: &str = include_str!("amount_cases.json");

    fn same(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
            (Value::Array(x), Value::Array(y)) => {
                x.len() == y.len() && x.iter().zip(y).all(|(l, r)| same(l, r))
            }
            (Value::Object(x), Value::Object(y)) => {
                x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w)))
            }
            _ => a == b,
        }
    }

    #[test]
    fn shared_cases_match() {
        let cases: Vec<Value> = serde_json::from_str(CASES).unwrap();
        for case in &cases {
            let name = case["name"].as_str().unwrap();
            let preset: Edits = serde_json::from_value(case["preset"].clone()).unwrap();
            let scaled =
                serde_json::to_value(preset.with_look_amount(case["amount"].as_f64().unwrap()))
                    .unwrap();
            for (pointer, want) in case["expect"].as_object().unwrap() {
                let got = scaled.pointer(pointer).unwrap_or(&Value::Null);
                assert!(same(got, want), "{name} {pointer}: got {got}, want {want}");
            }
        }
    }

    #[test]
    fn keeps_structure_outside_the_look() {
        let mut preset = Edits::default();
        preset.geometry.rotate = 1;
        preset.lens.profile_enabled = Some(true);
        preset.basic.contrast = 40.0;
        let scaled = preset.with_look_amount(50.0);
        assert_eq!(scaled.geometry, preset.geometry);
        assert_eq!(scaled.lens, preset.lens);
        assert_eq!(scaled.basic.contrast, 20.0);
    }
}
