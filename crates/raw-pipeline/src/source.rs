mod header;
#[cfg(feature = "native")]
mod window;
mod wire;

use crate::frame::FrameMeta;

#[cfg(feature = "native")]
pub use window::{WindowRect, crop_rgb, window_rect};
pub use wire::decode;
#[cfg(feature = "native")]
pub use wire::encode;

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

impl SourceWindow {
    pub fn uv_rect(&self, dims: (u32, u32)) -> [f32; 4] {
        let (fw, fh) = (self.full.0 as f32, self.full.1 as f32);
        [
            self.origin.0 as f32 / fw,
            self.origin.1 as f32 / fh,
            dims.0 as f32 / fw,
            dims.1 as f32 / fh,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct SourceHeader {
    pub meta: FrameMeta,
    pub kind: LinearKind,
    pub dims: (u32, u32),
    pub atmosphere: Option<[f32; 3]>,
    pub window: Option<SourceWindow>,
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
