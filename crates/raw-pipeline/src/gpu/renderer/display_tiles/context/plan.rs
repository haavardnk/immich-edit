use crate::gpu::helpers::mip_count;
use crate::gpu::renderer::detail::{atmosphere_level, haze_dims, level_dims};
use crate::ops::dehaze::DehazeGrid;
use crate::ops::presence::{PresenceBlurs, presence_blurs};
use crate::source::ContextNeeds;
use crate::tile::{Rect, support};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::gpu::renderer) enum Coarse {
    Texture,
    Clarity,
    Shadows,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::gpu::renderer) struct ContextPlan {
    pub needs: ContextNeeds,
    pub working: (u32, u32),
    pub haze: DehazeGrid,
    pub blurs: PresenceBlurs,
    pub atmosphere: u32,
    pub align: u32,
    texture: u32,
    clarity: u32,
    shadows: u32,
}

impl ContextPlan {
    pub fn new(needs: ContextNeeds, working: (u32, u32), align: u32) -> Option<Self> {
        if !needs.dehaze && !needs.coarse() {
            return None;
        }
        let blurs = presence_blurs(working.0, working.1);
        let mips = mip_count(working.0, working.1);
        let top = blurs.levels().min(mips) - 1;
        let haze = DehazeGrid::for_dims(working);
        let atmosphere = atmosphere_level(working).min(mips - 1);
        let texture = blurs.texture.level.min(top);
        let clarity = blurs.clarity.level.min(top);
        let shadows = blurs.shadows.level.min(top);
        let span = [
            (needs.dehaze, haze.scale.max(1 << atmosphere)),
            (needs.texture, 1 << texture),
            (needs.clarity, 1 << clarity),
            (needs.shadows, 1 << shadows),
        ]
        .into_iter()
        .filter_map(|(on, span)| on.then_some(span))
        .max()
        .unwrap_or(1);
        Some(Self {
            needs,
            working,
            haze,
            blurs,
            atmosphere,
            align: support::lcm(align, span),
            texture,
            clarity,
            shadows,
        })
    }

    pub fn cores(&self, edge: u32) -> Vec<Rect> {
        let edge = edge.div_ceil(self.align).max(1) * self.align;
        Rect::grid(self.working, edge).collect()
    }

    pub fn level(&self, coarse: Coarse) -> u32 {
        match coarse {
            Coarse::Texture => self.texture,
            Coarse::Clarity => self.clarity,
            Coarse::Shadows => self.shadows,
        }
    }

    pub fn collected(&self, adjusted: bool) -> Vec<Coarse> {
        let needs = self.needs;
        [
            (needs.texture && !adjusted, Coarse::Texture),
            (needs.clarity && !adjusted, Coarse::Clarity),
            (needs.shadows && needs.presence == adjusted, Coarse::Shadows),
        ]
        .into_iter()
        .filter_map(|(on, coarse)| on.then_some(coarse))
        .collect()
    }

    pub fn coarse(&self) -> impl Iterator<Item = Coarse> + '_ {
        [false, true]
            .into_iter()
            .flat_map(|adjusted| self.collected(adjusted))
    }

    pub fn lo(&self) -> (u32, u32) {
        haze_dims(self.working, self.haze.scale)
    }

    pub fn level_dims(&self, level: u32) -> (u32, u32) {
        level_dims(self.working, level)
    }

    pub fn fits(&self, limit: u32) -> bool {
        let (w, h) = self.working;
        let scale = self.haze.scale;
        let haze = [
            (w / scale, h / scale),
            (w >> self.atmosphere, h >> self.atmosphere),
        ]
        .into_iter()
        .filter(|_| self.needs.dehaze);
        let levels = self.coarse().map(|coarse| {
            let level = self.level(coarse);
            (w >> level, h >> level)
        });
        haze.chain(levels)
            .all(|(lw, lh)| lw > 0 && lh > 0 && lw.max(lh) <= limit)
    }
}
