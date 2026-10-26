pub mod coc;
pub mod kernel;

use super::{GpuPass, GpuRoute, Op, Stage};
use crate::edits::{Edits, LensBlurEdits};
use crate::mask_raster::{MaskRaster, RasterMap};
use coc::CocTable;
use kernel::BlurScale;
use std::sync::Arc;

pub struct LensBlurOp;

impl Op for LensBlurOp {
    fn id(&self) -> &'static str {
        "lens_blur"
    }
    fn gpu_route(&self) -> GpuRoute {
        GpuRoute::Pass(GpuPass::LensBlur)
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        -33
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.lens_blur.active()
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.lens_blur.is_default() {
            return None;
        }
        serde_json::to_value(&edits.lens_blur).ok()
    }
    fn apply_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Ok(lens_blur) = serde_json::from_value::<LensBlurEdits>(value.clone()) {
            edits.lens_blur = lens_blur;
        }
    }
}

pub struct LensBlur {
    pub raster_id: String,
    pub depth: Arc<MaskRaster>,
    pub table: CocTable,
    pub scale: BlurScale,
    pub blades: u32,
    pub boost: f32,
}

impl LensBlur {
    pub fn resolve(edits: &Edits, rasters: &RasterMap, dims: (u32, u32)) -> Option<Self> {
        let e = edits.lens_blur.clamped();
        if !e.active() {
            return None;
        }
        let depth = rasters.get(&e.depth_raster_id)?.clone();
        let table = CocTable::new(&depth, &e);
        Some(Self {
            raster_id: e.depth_raster_id.clone(),
            depth,
            table,
            scale: BlurScale::new(e.amount as f32, dims),
            blades: e.shape.blades(),
            boost: e.boost as f32 / 100.0,
        })
    }
}

pub fn window_margin(edits: &Edits, full: (u32, u32)) -> u32 {
    if edits.lens_blur.depth_raster_id.is_empty() {
        return 0;
    }
    BlurScale::new(100.0, full).margin()
}
