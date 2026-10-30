mod context;
mod header;
#[cfg(feature = "native")]
mod window;
pub(crate) mod wire;

use crate::frame::FrameMeta;

pub(crate) use context::ContextNeeds;
pub use context::{ContextGrid, ContextPart, SourceContext, context_key, window_key};
#[cfg(feature = "native")]
pub(crate) use context::{ContextPlane, ContextPlanes};
#[cfg(feature = "native")]
pub(crate) use window::source_bounds;
#[cfg(feature = "native")]
pub use window::{WindowRect, crop_rgb, window_rect};
pub use wire::decode;
#[cfg(feature = "native")]
pub use wire::encode;

#[cfg(feature = "native")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceCoding {
    Framed,
    ZstdContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinearKind {
    PreWb,
    PostWb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceWindow {
    pub origin: (u32, u32),
    pub full: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct SourceHeader {
    pub meta: FrameMeta,
    pub kind: LinearKind,
    pub dims: (u32, u32),
    pub atmosphere: Option<[f32; 3]>,
    pub window: Option<SourceWindow>,
    pub context: Option<SourceContext>,
}

#[derive(Debug, Clone)]
pub struct SourceImage {
    pub header: SourceHeader,
    pub rgb_f16: Vec<u16>,
    pub context: Vec<f32>,
}

#[cfg(feature = "native")]
pub struct RenderedSource {
    pub image: SourceImage,
    pub renderer: String,
    pub timings: Vec<crate::timing::StageTiming>,
    pub tiling: crate::tile::TilingReport,
}
