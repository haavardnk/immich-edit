mod header;
mod wire;

use crate::frame::FrameMeta;

pub use wire::decode;
#[cfg(feature = "native")]
pub use wire::encode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinearKind {
    PreWb,
    PostWb,
}

#[derive(Debug, Clone)]
pub struct SourceHeader {
    pub meta: FrameMeta,
    pub kind: LinearKind,
    pub dims: (u32, u32),
    pub atmosphere: Option<[f32; 3]>,
}

#[derive(Debug, Clone)]
pub struct SourceImage {
    pub header: SourceHeader,
    pub rgb_f16: Vec<u16>,
}

#[cfg(feature = "native")]
pub struct RenderedSource {
    pub image: SourceImage,
    pub renderer: String,
    pub timings: Vec<crate::timing::StageTiming>,
}
