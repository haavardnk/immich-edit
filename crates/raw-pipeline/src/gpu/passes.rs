pub mod demosaic;
pub mod mipgen;
pub mod process;

use std::sync::Arc;

use super::context::GpuContext;
use crate::ops::{OpRegistry, default_registry};

use demosaic::DemosaicPass;
use mipgen::MipgenPass;
use process::ProcessFastPass;

pub struct GpuPasses {
    pub demosaic: DemosaicPass,
    pub mipgen: MipgenPass,
    pub process_fast: ProcessFastPass,
    pub registry: OpRegistry,
}

impl GpuPasses {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let registry = default_registry();
        let demosaic = DemosaicPass::new(ctx);
        let mipgen = MipgenPass::new(ctx);
        let process_fast = ProcessFastPass::new(ctx, &registry);
        Self {
            demosaic,
            mipgen,
            process_fast,
            registry,
        }
    }
}
