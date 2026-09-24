use std::collections::HashMap;
use std::sync::Arc;

use wgpu::Texture;

use crate::edits::Edits;
use crate::frame::{FrameMeta, RawFrame};

pub type WbKey = (u64, u64);

pub fn wb_key(edits: &Edits) -> WbKey {
    (edits.basic.wb_temp.to_bits(), edits.basic.wb_tint.to_bits())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinearKind {
    PreWb,
    PostWb,
}

#[derive(Clone)]
pub struct LinearImage {
    pub texture: Arc<Texture>,
    pub atmosphere: Option<[f32; 3]>,
}

#[derive(Clone)]
pub struct LinearSource {
    pub meta: FrameMeta,
    pub kind: LinearKind,
    pub dims: (u32, u32),
    pub base: LinearImage,
    pub layer_bases: HashMap<WbKey, LinearImage>,
}

#[derive(Clone, Copy)]
pub enum RenderSource<'a> {
    Raw(&'a RawFrame),
    Linear(&'a LinearSource),
}

impl RenderSource<'_> {
    pub fn meta(&self) -> &FrameMeta {
        match self {
            Self::Raw(frame) => &frame.meta,
            Self::Linear(source) => &source.meta,
        }
    }
}

impl<'a> From<&'a RawFrame> for RenderSource<'a> {
    fn from(frame: &'a RawFrame) -> Self {
        Self::Raw(frame)
    }
}

impl<'a> From<&'a LinearSource> for RenderSource<'a> {
    fn from(source: &'a LinearSource) -> Self {
        Self::Linear(source)
    }
}
