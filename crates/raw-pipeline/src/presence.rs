use crate::edits::Edits;
use crate::gpu::passes::luma_pyramid::pyramid_levels_for;
use crate::gpu::passes::presence::select_mip;

const REFERENCE_DIM: f32 = 1080.0;

#[derive(Clone, Copy, Debug)]
pub struct PresenceAmounts {
    pub texture: f32,
    pub clarity: f32,
    pub dehaze: f32,
}

impl PresenceAmounts {
    pub fn is_zero(&self) -> bool {
        self.texture == 0.0 && self.clarity == 0.0 && self.dehaze == 0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PresenceRadii {
    pub texture: u32,
    pub clarity: u32,
    pub dehaze: u32,
    pub shadows: u32,
}

impl PresenceRadii {
    pub fn max(&self) -> u32 {
        self.texture
            .max(self.clarity)
            .max(self.dehaze)
            .max(self.shadows)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PresenceMips {
    pub texture: u32,
    pub clarity: u32,
    pub dehaze: u32,
    pub shadows: u32,
}

pub fn presence_amounts(edits: &Edits) -> PresenceAmounts {
    let t = (edits.basic.texture as f32 / 100.0).clamp(-1.0, 1.0);
    let c = (edits.basic.clarity as f32 / 100.0).clamp(-1.0, 1.0);
    PresenceAmounts {
        texture: t * 2.0,
        clarity: c * 1.0,
        dehaze: 0.0,
    }
}

pub fn presence_radii(width: u32, height: u32) -> PresenceRadii {
    let min_edge = width.min(height) as f32;
    let scale = (min_edge / REFERENCE_DIM).max(0.5);
    let texture = ((6.0 * scale).round() as u32).max(3);
    let clarity = ((30.0 * scale).round() as u32).max(10);
    let dehaze = ((120.0 * scale).round() as u32).max(20);
    let shadows = ((30.0 * scale).round() as u32).max(10);
    PresenceRadii {
        texture,
        clarity,
        dehaze,
        shadows,
    }
}

pub fn presence_mips(width: u32, height: u32, radii: PresenceRadii) -> PresenceMips {
    let max_edge = width.max(height);
    PresenceMips {
        texture: select_mip(max_edge, radii.texture),
        clarity: select_mip(max_edge, radii.clarity),
        dehaze: select_mip(max_edge, radii.dehaze),
        shadows: select_mip(max_edge, radii.shadows),
    }
}

pub fn has_shadows(edits: &Edits) -> bool {
    edits.tone.shadows != 0.0
}

pub fn presence_pyramid_levels(width: u32, height: u32, radii: PresenceRadii) -> u32 {
    pyramid_levels_for(width, height, radii.max())
}

pub fn has_presence(edits: &Edits) -> bool {
    !presence_amounts(edits).is_zero()
}
