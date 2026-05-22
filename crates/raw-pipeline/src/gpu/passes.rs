pub mod demosaic;
pub mod luma_pyramid;
pub mod mipgen;
pub mod presence;
pub mod process;

use std::sync::Arc;

use super::context::GpuContext;
use crate::ops::{OpRegistry, default_registry};

use demosaic::DemosaicPass;
use luma_pyramid::LumaPyramidPass;
use mipgen::MipgenPass;
use presence::PresencePass;
use process::ProcessFastPass;

pub struct GpuPasses {
    pub demosaic: DemosaicPass,
    pub mipgen: MipgenPass,
    pub luma_pyramid: LumaPyramidPass,
    pub presence: PresencePass,
    pub process_fast: ProcessFastPass,
    pub registry: OpRegistry,
}

impl GpuPasses {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let registry = default_registry();
        let demosaic = DemosaicPass::new(ctx);
        let mipgen = MipgenPass::new(ctx);
        let luma_pyramid = LumaPyramidPass::new(ctx);
        let presence = PresencePass::new(ctx);
        let process_fast = ProcessFastPass::new(ctx, &registry);
        Self {
            demosaic,
            mipgen,
            luma_pyramid,
            presence,
            process_fast,
            registry,
        }
    }
}
