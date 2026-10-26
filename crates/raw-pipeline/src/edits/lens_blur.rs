use serde::{Deserialize, Serialize};

pub(super) const LENS_BLUR_FOCUS_MAX: f64 = 0.3;

fn focus_max_default() -> f64 {
    LENS_BLUR_FOCUS_MAX
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum BokehShape {
    #[default]
    Circle,
    Hexagon,
    Octagon,
}

impl BokehShape {
    pub fn blades(self) -> u32 {
        match self {
            Self::Circle => 0,
            Self::Hexagon => 6,
            Self::Octagon => 8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LensBlurEdits {
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub focus_min: f64,
    #[serde(default = "focus_max_default")]
    pub focus_max: f64,
    #[serde(default)]
    pub shape: BokehShape,
    #[serde(default)]
    pub boost: f64,
    #[serde(default)]
    pub depth_raster_id: String,
}

impl Default for LensBlurEdits {
    fn default() -> Self {
        Self {
            amount: 0.0,
            focus_min: 0.0,
            focus_max: LENS_BLUR_FOCUS_MAX,
            shape: BokehShape::Circle,
            boost: 0.0,
            depth_raster_id: String::new(),
        }
    }
}

impl LensBlurEdits {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    pub fn active(&self) -> bool {
        self.amount > 0.0 && !self.depth_raster_id.is_empty()
    }

    pub fn clamped(&self) -> Self {
        let mut out = Self {
            amount: self.amount.clamp(0.0, 100.0),
            focus_min: self.focus_min.clamp(0.0, 1.0),
            focus_max: self.focus_max.clamp(0.0, 1.0),
            shape: self.shape,
            boost: self.boost.clamp(0.0, 100.0),
            depth_raster_id: self.depth_raster_id.clone(),
        };
        if out.focus_min > out.focus_max {
            std::mem::swap(&mut out.focus_min, &mut out.focus_max);
        }
        if out.amount == 0.0 {
            out = Self {
                depth_raster_id: out.depth_raster_id,
                ..Self::default()
            };
        }
        out
    }
}
