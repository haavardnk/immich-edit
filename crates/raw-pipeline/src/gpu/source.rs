use std::sync::Arc;

use wgpu::Texture;

use crate::frame::FrameMeta;
#[cfg(feature = "native")]
use crate::frame::RawFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinearKind {
    PreWb,
    PostWb,
}

#[derive(Clone)]
pub struct LinearSource {
    pub meta: FrameMeta,
    pub kind: LinearKind,
    pub dims: (u32, u32),
    pub texture: Arc<Texture>,
    pub atmosphere: Option<[f32; 3]>,
}

#[cfg(feature = "native")]
#[derive(Clone, Copy)]
pub enum RenderSource<'a> {
    Raw(&'a RawFrame),
    Linear(&'a LinearSource),
}

#[cfg(feature = "native")]
impl RenderSource<'_> {
    pub fn meta(&self) -> &FrameMeta {
        match self {
            Self::Raw(frame) => &frame.meta,
            Self::Linear(source) => &source.meta,
        }
    }
}

#[cfg(feature = "native")]
impl<'a> From<&'a RawFrame> for RenderSource<'a> {
    fn from(frame: &'a RawFrame) -> Self {
        Self::Raw(frame)
    }
}

#[cfg(feature = "native")]
impl<'a> From<&'a LinearSource> for RenderSource<'a> {
    fn from(source: &'a LinearSource) -> Self {
        Self::Linear(source)
    }
}
