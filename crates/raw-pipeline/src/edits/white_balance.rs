use serde::{Deserialize, Serialize};

use crate::color::Temperature;

const RELATIVE_LIMIT: f64 = 100.0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum WhiteBalance {
    Relative {
        #[serde(default)]
        temp: f64,
        #[serde(default)]
        tint: f64,
    },
    Absolute {
        kelvin: f64,
        #[serde(default)]
        tint: f64,
    },
}

impl Default for WhiteBalance {
    fn default() -> Self {
        Self::AS_SHOT
    }
}

impl From<Temperature> for WhiteBalance {
    fn from(t: Temperature) -> Self {
        Self::Absolute {
            kelvin: t.kelvin,
            tint: t.tint,
        }
    }
}

impl WhiteBalance {
    pub const AS_SHOT: Self = Self::Relative {
        temp: 0.0,
        tint: 0.0,
    };

    pub fn is_as_shot(&self) -> bool {
        *self == Self::AS_SHOT
    }

    pub fn clamped(&self) -> Self {
        match *self {
            Self::Relative { temp, tint } => Self::Relative {
                temp: temp.clamp(-RELATIVE_LIMIT, RELATIVE_LIMIT),
                tint: tint.clamp(-RELATIVE_LIMIT, RELATIVE_LIMIT),
            },
            Self::Absolute { kelvin, tint } => Temperature { kelvin, tint }.clamped().into(),
        }
    }

    pub fn shifted(&self, temp: f64, tint: f64) -> Self {
        match *self {
            Self::Relative { temp: t, tint: n } => Self::Relative {
                temp: (t + temp).clamp(-RELATIVE_LIMIT, RELATIVE_LIMIT),
                tint: (n + tint).clamp(-RELATIVE_LIMIT, RELATIVE_LIMIT),
            },
            Self::Absolute { kelvin, tint: n } => {
                Temperature { kelvin, tint: n }.shifted(temp, tint).into()
            }
        }
    }

    pub fn resolve(&self, as_shot: Temperature) -> Temperature {
        match *self {
            Self::Relative { temp, tint } => as_shot.shifted(temp, tint),
            Self::Absolute { kelvin, tint } => Temperature { kelvin, tint }.clamped(),
        }
    }
}

#[cfg(test)]
mod tests;
