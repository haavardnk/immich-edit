use serde::{Deserialize, Serialize};

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

impl GeometryEdits {
    pub fn clamped(&self) -> Self {
        Self {
            rotate: match self.rotate {
                0 | 90 | 180 | 270 => self.rotate,
                _ => 0,
            },
            rotate_angle: self.rotate_angle.clamp(-45.0, 45.0),
            flip_h: self.flip_h,
            flip_v: self.flip_v,
            crop: self.crop.map(|c| c.clamped()),
            aspect: self.aspect,
        }
    }
}
