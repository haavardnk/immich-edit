use crate::edits::Edits;
use crate::math::smoothstep;
use crate::tone::srgb_oetf;

const REFERENCE_DIM: f32 = 1080.0;
const TEXTURE_SIGMA: f32 = 4.0;
const CLARITY_SIGMA: f32 = 16.0;
const SHADOWS_SIGMA: f32 = 16.0;
const LEVEL_SIGMA: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresenceAmounts {
    pub texture: f32,
    pub clarity: f32,
    pub exposure: f32,
}

impl PresenceAmounts {
    pub fn is_zero(&self) -> bool {
        self.texture == 0.0 && self.clarity == 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresenceBlur {
    pub level: u32,
    pub sigma: f32,
}

impl PresenceBlur {
    fn new(sigma: f32, min_level: u32, top: u32) -> Self {
        let level = ((sigma / LEVEL_SIGMA).log2().floor().max(0.0) as u32)
            .max(min_level)
            .min(top);
        let scaled = sigma / (1u32 << level) as f32;
        let carried = (1.0 - 0.25f32.powi(level as i32)) / 12.0 + 1.0 / 6.0;
        Self {
            level,
            sigma: (scaled * scaled - carried).max(0.0).sqrt().max(0.01),
        }
    }

    pub fn radius(&self) -> u32 {
        (3.0 * self.sigma).ceil() as u32
    }

    fn reach(&self) -> u32 {
        (self.radius() + 2) << self.level
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresenceBlurs {
    pub texture: PresenceBlur,
    pub clarity: PresenceBlur,
    pub shadows: PresenceBlur,
}

impl PresenceBlurs {
    pub fn levels(&self) -> u32 {
        self.texture
            .level
            .max(self.clarity.level)
            .max(self.shadows.level)
            + 1
    }

    pub fn reach(&self) -> u32 {
        self.texture
            .reach()
            .max(self.clarity.reach())
            .max(self.shadows.reach())
    }
}

pub fn presence_amounts(edits: &Edits) -> PresenceAmounts {
    let t = (edits.basic.texture as f32 / 100.0).clamp(-1.0, 1.0);
    let c = (edits.basic.clarity as f32 / 100.0).clamp(-1.0, 1.0);
    PresenceAmounts {
        texture: t * 2.0,
        clarity: c * 1.0,
        exposure: if c == 0.0 {
            1.0
        } else {
            2f32.powf(edits.basic.exposure_ev as f32)
        },
    }
}

pub fn clarity_midtones(y: f32, exposure: f32) -> f32 {
    let d = srgb_oetf(y * exposure);
    smoothstep(0.0, 0.1, d) * (1.0 - smoothstep(0.9, 1.0, d)) * (1.0 - (2.0 * d - 1.0).abs())
}

pub fn presence_blurs(width: u32, height: u32) -> PresenceBlurs {
    let scale = width.min(height) as f32 / REFERENCE_DIM;
    let top = (width.max(height).max(1) as f32).log2().floor() as u32;
    PresenceBlurs {
        texture: PresenceBlur::new(TEXTURE_SIGMA * scale, 1, top),
        clarity: PresenceBlur::new(CLARITY_SIGMA * scale, 2, top),
        shadows: PresenceBlur::new(SHADOWS_SIGMA * scale, 1, top),
    }
}

pub fn has_shadows(edits: &Edits) -> bool {
    edits.tone.shadows != 0.0
        || edits
            .masks
            .iter()
            .filter(|l| l.is_effective())
            .any(|l| l.edits.shadows.is_some_and(|v| v != 0.0))
}

pub fn has_presence(edits: &Edits) -> bool {
    !presence_amounts(edits).is_zero()
}
