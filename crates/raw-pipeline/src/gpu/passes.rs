pub mod capture_sharpen;
pub mod common;
pub mod dcp_huesat;
pub mod dehaze;
pub mod demosaic;
pub mod effects_tone;
pub mod luma_pyramid;
pub mod lut;
pub mod mask_blend;
pub mod mask_overlay;
pub mod mask_weight;
pub mod mipgen;
pub mod nr;
pub mod nr_smooth;
pub mod presence;
pub mod process;
pub mod resample;
pub mod retouch;
pub mod sensor;
pub mod sharpen;
pub mod wb_prepare;
pub mod xtrans;

use std::sync::Arc;

use wgpu::{AddressMode, FilterMode, MipmapFilterMode, Sampler, SamplerDescriptor};

use super::context::GpuContext;
use crate::gpu::shader_builder::StageMask;
use crate::ops::{OpRegistry, default_registry};

use capture_sharpen::CaptureSharpenPasses;
use dcp_huesat::DcpHueSatPass;
use dehaze::DehazePasses;
use demosaic::DemosaicPass;
use effects_tone::EffectsTonePass;
use luma_pyramid::LumaPyramidPass;
use lut::LutPass;
use mask_blend::MaskBlendPass;
use mask_overlay::MaskOverlayPass;
use mask_weight::MaskWeightPass;
use mipgen::MipgenPass;
use nr::NrPass;
use nr_smooth::NrSmoothPass;
use presence::PresencePass;
use process::ProcessFastPass;
use resample::ResamplePass;
use retouch::RetouchPasses;
use sensor::SensorPass;
use sharpen::OutputSharpenPass;
use wb_prepare::WbPreparePass;
use xtrans::XtransPasses;

pub struct GpuPasses {
    pub dehaze: DehazePasses,
    pub demosaic: DemosaicPass,
    pub xtrans: XtransPasses,
    pub mipgen: MipgenPass,
    pub luma_pyramid: LumaPyramidPass,
    pub nr: NrPass,
    pub nr_smooth: NrSmoothPass,
    pub capture_sharpen: CaptureSharpenPasses,
    pub presence: PresencePass,
    pub resample: ResamplePass,
    pub retouch: RetouchPasses,
    pub wb_prepare: WbPreparePass,
    pub process_fast: ProcessFastPass,
    pub process_post_wb: ProcessFastPass,
    pub output_sharpen: OutputSharpenPass,
    pub effects_tone: EffectsTonePass,
    pub lut: LutPass,
    pub dcp_huesat: DcpHueSatPass,
    pub dcp_look: DcpHueSatPass,
    pub mask_weight: MaskWeightPass,
    pub mask_blend: MaskBlendPass,
    pub mask_overlay: MaskOverlayPass,
    pub sensor: SensorPass,
    pub linear_sampler: Sampler,
    pub atlas_sampler: Sampler,
    pub registry: OpRegistry,
}

impl GpuPasses {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let registry = default_registry();
        let (
            dehaze,
            demosaic,
            xtrans,
            mipgen,
            luma_pyramid,
            nr,
            nr_smooth,
            capture_sharpen,
            presence,
            resample,
            retouch,
            wb_prepare,
            process_fast,
            process_post_wb,
            output_sharpen,
            effects_tone,
            lut,
            dcp_huesat,
            dcp_look,
            mask_weight,
            mask_blend,
            mask_overlay,
            sensor,
        ) = std::thread::scope(|s| {
            let dehaze_t = s.spawn(|| DehazePasses::new(ctx));
            let demosaic_t = s.spawn(|| DemosaicPass::new(ctx));
            let xtrans_t = s.spawn(|| XtransPasses::new(ctx));
            let mipgen_t = s.spawn(|| MipgenPass::new(ctx));
            let luma_pyramid_t = s.spawn(|| LumaPyramidPass::new(ctx));
            let nr_t = s.spawn(|| NrPass::new(ctx));
            let nr_smooth_t = s.spawn(|| NrSmoothPass::new(ctx));
            let capture_sharpen_t = s.spawn(|| CaptureSharpenPasses::new(ctx));
            let presence_t = s.spawn(|| PresencePass::new(ctx));
            let resample_t = s.spawn(|| ResamplePass::new(ctx));
            let retouch_t = s.spawn(|| RetouchPasses::new(ctx));
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
            let effects_tone_t = s.spawn(|| EffectsTonePass::new(ctx));
            let lut_t = s.spawn(|| LutPass::new(ctx));
            let dcp_huesat_t = s.spawn(|| DcpHueSatPass::new(ctx));
            let dcp_look_t = s.spawn(|| DcpHueSatPass::new_look(ctx));
            let mask_weight_t = s.spawn(|| MaskWeightPass::new(ctx));
            let mask_blend_t = s.spawn(|| MaskBlendPass::new(ctx));
            let mask_overlay_t = s.spawn(|| MaskOverlayPass::new(ctx));
            let sensor_t = s.spawn(|| SensorPass::new(ctx));
            (
                dehaze_t.join().expect("dehaze pass build"),
                demosaic_t.join().expect("demosaic pass build"),
                xtrans_t.join().expect("xtrans pass build"),
                mipgen_t.join().expect("mipgen pass build"),
                luma_pyramid_t.join().expect("luma pyramid pass build"),
                nr_t.join().expect("nr pass build"),
                nr_smooth_t.join().expect("nr smooth pass build"),
                capture_sharpen_t
                    .join()
                    .expect("capture sharpen pass build"),
                presence_t.join().expect("presence pass build"),
                resample_t.join().expect("resample pass build"),
                retouch_t.join().expect("retouch pass build"),
                wb_prepare_t.join().expect("wb prepare pass build"),
                process_fast_t.join().expect("process fast pass build"),
                process_post_wb_t.join().expect("process post pass build"),
                output_sharpen_t.join().expect("output sharpen pass build"),
                effects_tone_t.join().expect("effects tone pass build"),
                lut_t.join().expect("lut pass build"),
                dcp_huesat_t.join().expect("dcp huesat pass build"),
                dcp_look_t.join().expect("dcp look pass build"),
                mask_weight_t.join().expect("mask weight pass build"),
                mask_blend_t.join().expect("mask blend pass build"),
                mask_overlay_t.join().expect("mask overlay pass build"),
                sensor_t.join().expect("sensor pass build"),
            )
        });
        Self {
            dehaze,
            demosaic,
            xtrans,
            mipgen,
            luma_pyramid,
            nr,
            nr_smooth,
            capture_sharpen,
            presence,
            resample,
            retouch,
            wb_prepare,
            process_fast,
            process_post_wb,
            output_sharpen,
            effects_tone,
            lut,
            dcp_huesat,
            dcp_look,
            mask_weight,
            mask_blend,
            mask_overlay,
            sensor,
            linear_sampler: ctx.device.create_sampler(&SamplerDescriptor {
                label: Some("linear-samp"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::Linear,
                ..Default::default()
            }),
            atlas_sampler: mask_weight::make_atlas_sampler(ctx),
            registry,
        }
    }
}
