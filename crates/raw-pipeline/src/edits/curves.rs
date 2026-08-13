use serde::{Deserialize, Serialize};

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

    pub fn clamped(&self) -> Self {
        Self {
            points: self
                .points
                .iter()
                .map(|p| CurvePoint {
                    x: p.x.clamp(0.0, 1.0),
                    y: p.y.clamp(0.0, 1.0),
                })
                .collect(),
        }
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
pub struct CurvesEdits {
    #[serde(default)]
    pub composite: CurvePoints,
    #[serde(default)]
    pub r: CurvePoints,
    #[serde(default)]
    pub g: CurvePoints,
    #[serde(default)]
    pub b: CurvePoints,
    #[serde(default)]
    pub luma: CurvePoints,
}

impl CurvesEdits {
    pub fn is_identity(&self) -> bool {
        self.composite.is_identity()
            && self.r.is_identity()
            && self.g.is_identity()
            && self.b.is_identity()
            && self.luma.is_identity()
    }

    pub fn clamped(&self) -> Self {
        Self {
            composite: self.composite.clamped(),
            r: self.r.clamped(),
            g: self.g.clamped(),
            b: self.b.clamped(),
            luma: self.luma.clamped(),
        }
    }
}
