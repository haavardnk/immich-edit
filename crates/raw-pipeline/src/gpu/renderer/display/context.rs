use wgpu::TextureView;

use crate::gpu::source::SourceExtent;
use crate::source::ContextNeeds;

#[derive(Clone, Copy)]
pub(in crate::gpu::renderer) struct Placed {
    pub extent: SourceExtent,
    pub origin: (u32, u32),
}

#[derive(Clone)]
pub(in crate::gpu) struct GridView {
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
pub(in crate::gpu) struct LevelView {
    pub grid: GridView,
    pub level: u32,
}

#[derive(Clone)]
pub(in crate::gpu) struct HazeGrid {
    pub atmosphere: [f32; 3],
    pub ab: GridView,
}

#[derive(Clone, Default)]
pub(in crate::gpu) struct PresenceLevels {
    pub texture: Option<LevelView>,
    pub guide: Option<LevelView>,
}

impl PresenceLevels {
    pub fn covers(&self, needs: ContextNeeds) -> bool {
        (!needs.texture || self.texture.is_some()) && (!needs.clarity || self.guide.is_some())
    }
}

pub(in crate::gpu::renderer) struct ShadowLevels {
    pub luma: TextureView,
    pub blur: TextureView,
    pub window: [u32; 4],
    pub origin: (u32, u32),
    pub blur_origin: (u32, u32),
    pub level: u32,
}

#[derive(Default)]
pub(in crate::gpu) struct GlobalContext {
    pub haze: Option<HazeGrid>,
    pub presence: PresenceLevels,
    pub shadows: Option<LevelView>,
}
