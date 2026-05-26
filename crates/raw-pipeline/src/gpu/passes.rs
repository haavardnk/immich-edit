pub mod dehaze;
pub mod demosaic;
pub mod luma_pyramid;
pub mod mask_blend;
pub mod mask_weight;
pub mod mipgen;
pub mod nr;
pub mod nr_smooth;
pub mod presence;
pub mod process;
pub mod sensor;
pub mod sharpen;
pub mod wb_prepare;

use std::sync::Arc;

use super::context::GpuContext;
use crate::gpu::shader_builder::StageMask;
use crate::ops::{OpRegistry, default_registry};

use dehaze::DehazePasses;
use demosaic::DemosaicPass;
use luma_pyramid::LumaPyramidPass;
use mask_blend::MaskBlendPass;
use mask_weight::MaskWeightPass;
use mipgen::MipgenPass;
use nr::NrPass;
use nr_smooth::NrSmoothPass;
use presence::PresencePass;
use process::ProcessFastPass;
use sensor::SensorPass;
use sharpen::OutputSharpenPass;
use wb_prepare::WbPreparePass;

pub struct GpuPasses {
    pub dehaze: DehazePasses,
    pub demosaic: DemosaicPass,
    pub mipgen: MipgenPass,
    pub luma_pyramid: LumaPyramidPass,
    pub nr: NrPass,
    pub nr_smooth: NrSmoothPass,
    pub presence: PresencePass,
    pub wb_prepare: WbPreparePass,
    pub process_fast: ProcessFastPass,
    pub process_post_wb: ProcessFastPass,
    pub output_sharpen: OutputSharpenPass,
    pub mask_weight: MaskWeightPass,
    pub mask_blend: MaskBlendPass,
    pub sensor: SensorPass,
    pub registry: OpRegistry,
}

impl GpuPasses {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let registry = default_registry();
        let dehaze = DehazePasses::new(ctx);
        let demosaic = DemosaicPass::new(ctx);
        let mipgen = MipgenPass::new(ctx);
        let luma_pyramid = LumaPyramidPass::new(ctx);
        let nr = NrPass::new(ctx);
        let nr_smooth = NrSmoothPass::new(ctx);
        let presence = PresencePass::new(ctx);
        let wb_prepare = WbPreparePass::new(ctx, &registry);
        let process_fast = ProcessFastPass::new(ctx, &registry);
        let process_post_wb =
            ProcessFastPass::new_with_mask(ctx, &registry, StageMask::tone_color(), "process-post");
        let output_sharpen = OutputSharpenPass::new(ctx);
        let mask_weight = MaskWeightPass::new(ctx);
        let mask_blend = MaskBlendPass::new(ctx);
        let sensor = SensorPass::new(ctx);
        Self {
            dehaze,
            demosaic,
            mipgen,
            luma_pyramid,
            nr,
            nr_smooth,
            presence,
            wb_prepare,
            process_fast,
            process_post_wb,
            output_sharpen,
            mask_weight,
            mask_blend,
            sensor,
            registry,
        }
    }
}
