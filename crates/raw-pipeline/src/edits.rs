use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const CURVE_LUT_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CurvePoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(transparent)]
pub struct CurvePoints {
    pub points: Vec<CurvePoint>,
}

impl<'de> Deserialize<'de> for CurvePoints {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;
        use serde_json::Value;

        let v = Value::deserialize(deserializer)?;
        let Value::Array(arr) = v else {
            return Err(de::Error::custom("expected array for curves"));
        };
        let pts: Vec<CurvePoint> = arr
            .into_iter()
            .filter_map(|item| serde_json::from_value(item).ok())
            .collect();
        if pts.len() >= 2 {
            Ok(Self { points: pts })
        } else {
            Ok(Self::default())
        }
    }
}

impl CurvePoints {
    fn default_points() -> Vec<CurvePoint> {
        vec![CurvePoint { x: 0.0, y: 0.0 }, CurvePoint { x: 1.0, y: 1.0 }]
    }

    pub fn is_identity(&self) -> bool {
        self.points.len() == 2
            && self.points[0].x.abs() < 1e-10
            && self.points[0].y.abs() < 1e-10
            && (self.points[1].x - 1.0).abs() < 1e-10
            && (self.points[1].y - 1.0).abs() < 1e-10
    }

    pub fn as_tuples(&self) -> Vec<(f64, f64)> {
        self.points.iter().map(|p| (p.x, p.y)).collect()
    }
}

impl Default for CurvePoints {
    fn default() -> Self {
        Self {
            points: Self::default_points(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BasicEdits {
    #[serde(default)]
    pub exposure_ev: f64,
    #[serde(default)]
    pub contrast: f64,
    #[serde(default)]
    pub saturation: f64,
    #[serde(default)]
    pub vibrance: f64,
    #[serde(default)]
    pub wb_temp: f64,
    #[serde(default)]
    pub wb_tint: f64,
    #[serde(default)]
    pub curves: CurvePoints,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToneEdits {
    #[serde(default)]
    pub highlights: f64,
    #[serde(default)]
    pub shadows: f64,
    #[serde(default)]
    pub blacks: f64,
    #[serde(default)]
    pub whites: f64,
}

pub const HSL_BANDS: usize = 8;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct HslBand {
    #[serde(default)]
    pub hue: f64,
    #[serde(default)]
    pub sat: f64,
    #[serde(default)]
    pub lum: f64,
}

impl HslBand {
    pub fn is_zero(&self) -> bool {
        self.hue == 0.0 && self.sat == 0.0 && self.lum == 0.0
    }

    pub fn clamped(&self) -> Self {
        Self {
            hue: self.hue.clamp(-100.0, 100.0),
            sat: self.sat.clamp(-100.0, 100.0),
            lum: self.lum.clamp(-100.0, 100.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HslEdits {
    #[serde(default = "default_bands")]
    pub bands: [HslBand; HSL_BANDS],
}

fn default_bands() -> [HslBand; HSL_BANDS] {
    [HslBand::default(); HSL_BANDS]
}

impl Default for HslEdits {
    fn default() -> Self {
        Self {
            bands: default_bands(),
        }
    }
}

impl HslEdits {
    pub fn is_zero(&self) -> bool {
        self.bands.iter().all(|b| b.is_zero())
    }

    pub fn clamped(&self) -> Self {
        let mut bands = [HslBand::default(); HSL_BANDS];
        for (i, b) in bands.iter_mut().enumerate() {
            *b = self.bands[i].clamped();
        }
        Self { bands }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct ColorGradeRegion {
    #[serde(default)]
    pub hue: f64,
    #[serde(default)]
    pub sat: f64,
    #[serde(default)]
    pub lum: f64,
}

impl ColorGradeRegion {
    pub fn is_zero(&self) -> bool {
        self.sat == 0.0 && self.lum == 0.0
    }

    pub fn clamped(&self) -> Self {
        Self {
            hue: self.hue.rem_euclid(360.0),
            sat: self.sat.clamp(0.0, 100.0),
            lum: self.lum.clamp(-50.0, 50.0),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct ColorGradeEdits {
    #[serde(default)]
    pub shadows: ColorGradeRegion,
    #[serde(default)]
    pub midtones: ColorGradeRegion,
    #[serde(default)]
    pub highlights: ColorGradeRegion,
    #[serde(default)]
    pub global: ColorGradeRegion,
    #[serde(default)]
    pub balance: f64,
    #[serde(default)]
    pub blend: f64,
}

impl ColorGradeEdits {
    pub fn is_zero(&self) -> bool {
        self.shadows.is_zero()
            && self.midtones.is_zero()
            && self.highlights.is_zero()
            && self.global.is_zero()
    }

    pub fn clamped(&self) -> Self {
        Self {
            shadows: self.shadows.clamped(),
            midtones: self.midtones.clamped(),
            highlights: self.highlights.clamped(),
            global: self.global.clamped(),
            balance: self.balance.clamp(-100.0, 100.0),
            blend: self.blend.clamp(0.0, 100.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ColorEdits {
    #[serde(default)]
    pub hsl: HslEdits,
    #[serde(default)]
    pub color_grade: ColorGradeEdits,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CropRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl CropRect {
    pub fn full() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        }
    }

    pub fn is_full(&self) -> bool {
        (self.x).abs() < 1e-5
            && (self.y).abs() < 1e-5
            && (self.w - 1.0).abs() < 1e-5
            && (self.h - 1.0).abs() < 1e-5
    }

    pub fn clamped(&self) -> Self {
        let x = self.x.clamp(0.0, 1.0);
        let y = self.y.clamp(0.0, 1.0);
        let w = self.w.clamp(0.0, 1.0 - x);
        let h = self.h.clamp(0.0, 1.0 - y);
        Self { x, y, w, h }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AspectLock {
    #[default]
    Original,
    Free,
    Ratio {
        num: u16,
        den: u16,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GeometryEdits {
    #[serde(default)]
    pub rotate: u16,
    #[serde(default)]
    pub rotate_angle: f32,
    #[serde(default)]
    pub flip_h: bool,
    #[serde(default)]
    pub flip_v: bool,
    #[serde(default)]
    pub crop: Option<CropRect>,
    #[serde(default)]
    pub aspect: AspectLock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Edits {
    #[serde(default)]
    pub basic: BasicEdits,
    #[serde(default)]
    pub tone: ToneEdits,
    #[serde(default)]
    pub color: ColorEdits,
    #[serde(default)]
    pub geometry: GeometryEdits,
}

impl Edits {
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }

    pub fn clamped(&self) -> Self {
        let rotate = match self.geometry.rotate {
            0 | 90 | 180 | 270 => self.geometry.rotate,
            _ => 0,
        };
        Self {
            basic: BasicEdits {
                exposure_ev: self.basic.exposure_ev.clamp(-5.0, 5.0),
                contrast: self.basic.contrast.clamp(-100.0, 100.0),
                saturation: self.basic.saturation.clamp(-100.0, 100.0),
                vibrance: self.basic.vibrance.clamp(-100.0, 100.0),
                wb_temp: self.basic.wb_temp.clamp(-100.0, 100.0),
                wb_tint: self.basic.wb_tint.clamp(-100.0, 100.0),
                curves: self.basic.curves.clone(),
            },
            tone: ToneEdits {
                highlights: self.tone.highlights.clamp(-100.0, 100.0),
                shadows: self.tone.shadows.clamp(-100.0, 100.0),
                blacks: self.tone.blacks.clamp(-100.0, 100.0),
                whites: self.tone.whites.clamp(-100.0, 100.0),
            },
            color: ColorEdits {
                hsl: self.color.hsl.clamped(),
                color_grade: self.color.color_grade.clamped(),
            },
            geometry: GeometryEdits {
                rotate,
                rotate_angle: self.geometry.rotate_angle.clamp(-45.0, 45.0),
                flip_h: self.geometry.flip_h,
                flip_v: self.geometry.flip_v,
                crop: self.geometry.crop.map(|c| c.clamped()),
                aspect: self.geometry.aspect,
            },
        }
    }

    pub fn stable_hash(&self) -> String {
        let clamped = self.clamped();
        let json = serde_json::to_string(&clamped).expect("edits serialize");
        let hash = Sha256::digest(json.as_bytes());
        hex::encode(&hash[..16])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_identity() {
        let e = Edits::default();
        assert!(e.is_identity());
    }

    #[test]
    fn clamp_exposure() {
        let mut e = Edits::default();
        e.basic.exposure_ev = 10.0;
        let c = e.clamped();
        assert_eq!(c.basic.exposure_ev, 5.0);
    }

    #[test]
    fn clamp_invalid_rotate() {
        let mut e = Edits::default();
        e.geometry.rotate = 45;
        let c = e.clamped();
        assert_eq!(c.geometry.rotate, 0);
    }

    #[test]
    fn stable_hash_deterministic() {
        let mut e = Edits::default();
        e.basic.exposure_ev = 1.5;
        let h1 = e.stable_hash();
        let h2 = e.stable_hash();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 32);
    }

    #[test]
    fn stable_hash_differs_on_change() {
        let mut a = Edits::default();
        a.basic.exposure_ev = 1.0;
        let mut b = Edits::default();
        b.basic.exposure_ev = 2.0;
        assert_ne!(a.stable_hash(), b.stable_hash());
    }

    #[test]
    fn serde_roundtrip() {
        let mut e = Edits::default();
        e.basic.exposure_ev = 1.0;
        e.geometry.rotate = 90;
        let json = serde_json::to_string(&e).unwrap();
        let e2: Edits = serde_json::from_str(&json).unwrap();
        assert_eq!(e, e2);
    }

    #[test]
    fn serde_defaults() {
        let json = "{}";
        let e: Edits = serde_json::from_str(json).unwrap();
        assert!(e.is_identity());
    }
}
