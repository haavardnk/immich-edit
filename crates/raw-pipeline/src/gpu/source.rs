use std::sync::Arc;

use wgpu::Texture;

use crate::frame::FrameMeta;
#[cfg(feature = "native")]
use crate::frame::RawFrame;
use crate::source::{LinearKind, SourceHeader, SourceWindow};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceExtent {
    pub dims: (u32, u32),
    pub full: (u32, u32),
}

#[derive(Clone)]
pub struct LinearSource {
    pub meta: FrameMeta,
    pub kind: LinearKind,
    pub dims: (u32, u32),
    pub texture: Arc<Texture>,
    pub atmosphere: Option<[f32; 3]>,
    pub window: Option<SourceWindow>,
}

impl LinearSource {
    pub fn header(&self) -> SourceHeader {
        SourceHeader {
            meta: self.meta.clone(),
            kind: self.kind,
            dims: self.dims,
            atmosphere: self.atmosphere,
            window: self.window,
        }
    }

    pub fn extent(&self) -> SourceExtent {
        SourceExtent {
            dims: self.dims,
            full: self.window.map_or(self.dims, |w| w.full),
        }
    }
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
