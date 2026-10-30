use std::iter::once;

use wgpu::TextureView;

use super::linear::layer_presence;
use crate::PipelineError;
use crate::edits::Edits;
use crate::gpu::source::SourceExtent;
use crate::ops::presence::{has_shadows, presence_amounts};

#[derive(Clone, Copy)]
pub(in crate::gpu::renderer) struct Placed {
    pub extent: SourceExtent,
    pub origin: (u32, u32),
}

#[derive(Clone)]
pub(in crate::gpu::renderer) struct GridView {
    pub view: TextureView,
    pub origin: (u32, u32),
}

impl GridView {
    pub fn local(view: TextureView, origin: (u32, u32), shift: u32) -> Self {
        Self {
            view,
            origin: (origin.0 >> shift, origin.1 >> shift),
        }
    }

    #[cfg(feature = "native")]
    pub fn global(view: TextureView) -> Self {
        Self {
            view,
            origin: (0, 0),
        }
    }
}

#[derive(Clone)]
pub(in crate::gpu::renderer) struct LevelView {
    pub grid: GridView,
    pub level: u32,
}

#[derive(Clone)]
pub(in crate::gpu::renderer) struct HazeGrid {
    pub atmosphere: [f32; 3],
    pub ab: GridView,
}

#[derive(Clone, Default)]
pub(in crate::gpu::renderer) struct PresenceLevels {
    pub texture: Option<LevelView>,
    pub guide: Option<LevelView>,
}

pub(in crate::gpu::renderer) struct ShadowLevels {
    pub luma: TextureView,
    pub blur: TextureView,
    pub window: [u32; 4],
    pub origin: (u32, u32),
    pub level: u32,
}

#[derive(Default)]
pub(in crate::gpu::renderer) struct GlobalContext {
    pub haze: Option<HazeGrid>,
    pub presence: PresenceLevels,
    pub shadows: Option<LevelView>,
}

impl GlobalContext {
    pub fn haze(&self) -> Result<&HazeGrid, PipelineError> {
        self.haze.as_ref().ok_or_else(|| missing("dehaze"))
    }

    pub fn shadows(&self) -> Result<&LevelView, PipelineError> {
        self.shadows.as_ref().ok_or_else(|| missing("shadows"))
    }
}

fn missing(stage: &str) -> PipelineError {
    PipelineError::Render(format!("display tile context has no {stage} levels"))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::gpu::renderer) struct ContextNeeds {
    pub dehaze: bool,
    pub texture: bool,
    pub clarity: bool,
    pub shadows: bool,
    pub presence: bool,
}

impl ContextNeeds {
    pub fn new(edits: &Edits) -> Self {
        let global = presence_amounts(edits);
        let layers = layer_presence(edits);
        let adjusting = once(global)
            .chain(layers.into_iter().map(|(_, _, amounts)| amounts))
            .filter(|amounts| amounts.adjusts());
        let (texture, clarity) = adjusting.fold((false, false), |(t, c), amounts| {
            (t || amounts.texture != 0.0, c || amounts.clarity != 0.0)
        });
        Self {
            dehaze: edits.basic.dehaze != 0.0,
            texture,
            clarity,
            shadows: has_shadows(edits),
            presence: global.adjusts(),
        }
    }

    pub fn levels(&self) -> bool {
        self.texture || self.clarity
    }
}
