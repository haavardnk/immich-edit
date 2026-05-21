use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct CropRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl CropRect {
    const MIN_DIM: f64 = 0.01;

    pub fn clamped(&self) -> Self {
        let x = self.x.clamp(0.0, 1.0);
        let y = self.y.clamp(0.0, 1.0);
        let width = self.width.clamp(Self::MIN_DIM, 1.0 - x);
        let height = self.height.clamp(Self::MIN_DIM, 1.0 - y);
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edits {
    #[serde(default)]
    pub exposure_ev: f64,
    #[serde(default)]
    pub contrast: f64,
    #[serde(default)]
    pub highlights: f64,
    #[serde(default)]
    pub shadows: f64,
    #[serde(default)]
    pub saturation: f64,
    #[serde(default)]
    pub wb_temp: f64,
    #[serde(default)]
    pub wb_tint: f64,
    #[serde(default)]
    pub rotate: u16,
    #[serde(default)]
    pub flip_h: bool,
    #[serde(default)]
    pub flip_v: bool,
    #[serde(default)]
    pub crop: Option<CropRect>,
}

impl Default for Edits {
    fn default() -> Self {
        Self {
            exposure_ev: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            saturation: 0.0,
            wb_temp: 0.0,
            wb_tint: 0.0,
            rotate: 0,
            flip_h: false,
            flip_v: false,
            crop: None,
        }
    }
}

impl Edits {
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }

    pub fn clamped(&self) -> Self {
        let rotate = match self.rotate {
            0 | 90 | 180 | 270 => self.rotate,
            _ => 0,
        };
        Self {
            exposure_ev: self.exposure_ev.clamp(-5.0, 5.0),
            contrast: self.contrast.clamp(-100.0, 100.0),
            highlights: self.highlights.clamp(-100.0, 100.0),
            shadows: self.shadows.clamp(-100.0, 100.0),
            saturation: self.saturation.clamp(-100.0, 100.0),
            wb_temp: self.wb_temp.clamp(-100.0, 100.0),
            wb_tint: self.wb_tint.clamp(-100.0, 100.0),
            rotate,
            flip_h: self.flip_h,
            flip_v: self.flip_v,
            crop: self.crop.as_ref().map(|c| c.clamped()),
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
        let e = Edits {
            exposure_ev: 10.0,
            ..Default::default()
        };
        let c = e.clamped();
        assert_eq!(c.exposure_ev, 5.0);
    }

    #[test]
    fn clamp_invalid_rotate() {
        let e = Edits {
            rotate: 45,
            ..Default::default()
        };
        let c = e.clamped();
        assert_eq!(c.rotate, 0);
    }

    #[test]
    fn clamp_crop() {
        let e = Edits {
            crop: Some(CropRect {
                x: -0.5,
                y: 0.5,
                width: 2.0,
                height: 0.001,
            }),
            ..Default::default()
        };
        let c = e.clamped();
        let crop = c.crop.unwrap();
        assert_eq!(crop.x, 0.0);
        assert_eq!(crop.y, 0.5);
        assert_eq!(crop.width, 1.0);
        assert!(crop.height >= CropRect::MIN_DIM);
    }

    #[test]
    fn stable_hash_deterministic() {
        let e = Edits {
            exposure_ev: 1.5,
            ..Default::default()
        };
        let h1 = e.stable_hash();
        let h2 = e.stable_hash();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 32);
    }

    #[test]
    fn stable_hash_differs_on_change() {
        let a = Edits {
            exposure_ev: 1.0,
            ..Default::default()
        };
        let b = Edits {
            exposure_ev: 2.0,
            ..Default::default()
        };
        assert_ne!(a.stable_hash(), b.stable_hash());
    }

    #[test]
    fn serde_roundtrip() {
        let e = Edits {
            exposure_ev: 1.0,
            rotate: 90,
            ..Default::default()
        };
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
