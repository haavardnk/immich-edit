pub mod demosaic;
pub mod luma_pyramid;
pub mod mipgen;
pub mod presence;
pub mod process;
pub mod wb_prepare;

use std::sync::Arc;

use super::context::GpuContext;
use crate::gpu::shader_builder::StageMask;
use crate::ops::{OpRegistry, default_registry};

use demosaic::DemosaicPass;
use luma_pyramid::LumaPyramidPass;
use mipgen::MipgenPass;
use presence::PresencePass;
use process::ProcessFastPass;
use wb_prepare::WbPreparePass;

pub struct GpuPasses {
    pub demosaic: DemosaicPass,
    pub mipgen: MipgenPass,
    pub luma_pyramid: LumaPyramidPass,
    pub presence: PresencePass,
    pub wb_prepare: WbPreparePass,
    pub process_fast: ProcessFastPass,
    pub process_post_wb: ProcessFastPass,
    pub registry: OpRegistry,
}

impl GpuPasses {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let registry = default_registry();
        let demosaic = DemosaicPass::new(ctx);
        let mipgen = MipgenPass::new(ctx);
        let luma_pyramid = LumaPyramidPass::new(ctx);
        let presence = PresencePass::new(ctx);
        let wb_prepare = WbPreparePass::new(ctx, &registry);
        let process_fast = ProcessFastPass::new(ctx, &registry);
        let process_post_wb =
            ProcessFastPass::new_with_mask(ctx, &registry, StageMask::tone_color(), "process-post");
        Self {
            demosaic,
            mipgen,
            luma_pyramid,
            presence,
            wb_prepare,
            process_fast,
            process_post_wb,
            registry,
        }
    }
}
