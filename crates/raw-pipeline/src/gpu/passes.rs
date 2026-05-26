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
        let (
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
        ) = std::thread::scope(|s| {
            let dehaze_t = s.spawn(|| DehazePasses::new(ctx));
            let demosaic_t = s.spawn(|| DemosaicPass::new(ctx));
            let mipgen_t = s.spawn(|| MipgenPass::new(ctx));
            let luma_pyramid_t = s.spawn(|| LumaPyramidPass::new(ctx));
            let nr_t = s.spawn(|| NrPass::new(ctx));
            let nr_smooth_t = s.spawn(|| NrSmoothPass::new(ctx));
            let presence_t = s.spawn(|| PresencePass::new(ctx));
            let wb_prepare_t = s.spawn(|| WbPreparePass::new(ctx, &registry));
            let process_fast_t = s.spawn(|| ProcessFastPass::new(ctx, &registry));
            let process_post_wb_t = s.spawn(|| {
                ProcessFastPass::new_with_mask(
                    ctx,
                    &registry,
                    StageMask::tone_color(),
                    "process-post",
                )
            });
            let output_sharpen_t = s.spawn(|| OutputSharpenPass::new(ctx));
            let mask_weight_t = s.spawn(|| MaskWeightPass::new(ctx));
            let mask_blend_t = s.spawn(|| MaskBlendPass::new(ctx));
            let sensor_t = s.spawn(|| SensorPass::new(ctx));
            (
                dehaze_t.join().expect("dehaze pass build"),
                demosaic_t.join().expect("demosaic pass build"),
                mipgen_t.join().expect("mipgen pass build"),
                luma_pyramid_t.join().expect("luma pyramid pass build"),
                nr_t.join().expect("nr pass build"),
                nr_smooth_t.join().expect("nr smooth pass build"),
                presence_t.join().expect("presence pass build"),
                wb_prepare_t.join().expect("wb prepare pass build"),
                process_fast_t.join().expect("process fast pass build"),
                process_post_wb_t.join().expect("process post pass build"),
                output_sharpen_t.join().expect("output sharpen pass build"),
                mask_weight_t.join().expect("mask weight pass build"),
                mask_blend_t.join().expect("mask blend pass build"),
                sensor_t.join().expect("sensor pass build"),
            )
        });
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
