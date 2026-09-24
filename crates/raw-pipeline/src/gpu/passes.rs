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
pub mod meta_bins;
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
use crate::gpu::display_depth::DisplayDepth;
use crate::gpu::shader_builder::StageMask;
use crate::ops::{OpRegistry, default_registry};

#[cfg(feature = "native")]
use capture_sharpen::CaptureSharpenPasses;
use dcp_huesat::DcpHueSatPass;
use dehaze::DehazePasses;
#[cfg(feature = "native")]
use demosaic::DemosaicPass;
use effects_tone::EffectsTonePass;
use luma_pyramid::LumaPyramidPass;
use lut::LutPass;
use mask_blend::MaskBlendPass;
use mask_overlay::MaskOverlayPass;
use mask_weight::MaskWeightPass;
use meta_bins::MetaBinsPasses;
use mipgen::MipgenPass;
#[cfg(feature = "native")]
use nr::NrPass;
#[cfg(feature = "native")]
use nr_smooth::NrSmoothPass;
use presence::PresencePass;
use process::ProcessFastPass;
use resample::ResamplePass;
#[cfg(feature = "native")]
use retouch::RetouchPasses;
#[cfg(feature = "native")]
use sensor::SensorPass;
use sharpen::OutputSharpenPass;
#[cfg(feature = "native")]
use wb_prepare::WbPreparePass;
#[cfg(feature = "native")]
use xtrans::XtransPasses;

macro_rules! build_passes {
    ($($field:ident: $build:expr),* $(,)?) => {
        #[cfg(feature = "native")]
        let ($($field,)*) = std::thread::scope(|s| {
            $(let $field = s.spawn(|| $build);)*
            ($($field.join().expect(concat!(stringify!($field), " pass build")),)*)
        });
        #[cfg(not(feature = "native"))]
        let ($($field,)*) = ($($build,)*);
    };
}

pub struct GpuPasses {
    pub dehaze: DehazePasses,
    pub mipgen: MipgenPass,
    pub luma_pyramid: LumaPyramidPass,
    pub presence: PresencePass,
    pub resample: ResamplePass,
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
    pub meta_bins: MetaBinsPasses,
    #[cfg(feature = "native")]
    pub sensor_stage: SensorStagePasses,
    pub linear_sampler: Sampler,
    pub atlas_sampler: Sampler,
    pub registry: OpRegistry,
    depth16: std::sync::OnceLock<Depth16Passes>,
}

#[cfg(feature = "native")]
pub struct SensorStagePasses {
    pub demosaic: DemosaicPass,
    pub xtrans: XtransPasses,
    pub sensor: SensorPass,
    pub wb_prepare: WbPreparePass,
    pub retouch: RetouchPasses,
    pub nr: NrPass,
    pub nr_smooth: NrSmoothPass,
    pub capture_sharpen: CaptureSharpenPasses,
}

#[cfg(feature = "native")]
impl SensorStagePasses {
    fn new(ctx: &Arc<GpuContext>, registry: &OpRegistry) -> Self {
        build_passes! {
            demosaic: DemosaicPass::new(ctx),
            xtrans: XtransPasses::new(ctx),
            sensor: SensorPass::new(ctx),
            wb_prepare: WbPreparePass::new(ctx, registry),
            retouch: RetouchPasses::new(ctx),
            nr: NrPass::new(ctx),
            nr_smooth: NrSmoothPass::new(ctx),
            capture_sharpen: CaptureSharpenPasses::new(ctx),
        }
        Self {
            demosaic,
            xtrans,
            sensor,
            wb_prepare,
            retouch,
            nr,
            nr_smooth,
            capture_sharpen,
        }
    }
}

pub struct Depth16Passes {
    pub process_fast: ProcessFastPass,
    pub process_post_wb: ProcessFastPass,
    pub effects_tone: EffectsTonePass,
    pub lut: LutPass,
    pub dcp_look: DcpHueSatPass,
    pub meta_bins: MetaBinsPasses,
}

impl GpuPasses {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let registry = default_registry();
        build_passes! {
            dehaze: DehazePasses::new(ctx),
            mipgen: MipgenPass::new(ctx),
            luma_pyramid: LumaPyramidPass::new(ctx),
            presence: PresencePass::new(ctx),
            resample: ResamplePass::new(ctx),
            process_fast: ProcessFastPass::new(ctx, &registry),
            process_post_wb: ProcessFastPass::new_with_mask(
                ctx,
                &registry,
                StageMask::tone_color(),
                DisplayDepth::Eight,
                "process-post",
            ),
            output_sharpen: OutputSharpenPass::new(ctx),
            effects_tone: EffectsTonePass::new(ctx, DisplayDepth::Eight),
            lut: LutPass::new(ctx, DisplayDepth::Eight),
            dcp_huesat: DcpHueSatPass::new(ctx),
            dcp_look: DcpHueSatPass::new_look(ctx, DisplayDepth::Eight.format()),
            mask_weight: MaskWeightPass::new(ctx),
            mask_blend: MaskBlendPass::new(ctx),
            mask_overlay: MaskOverlayPass::new(ctx),
            meta_bins: MetaBinsPasses::new(ctx, DisplayDepth::Eight),
        }
        Self {
            dehaze,
            mipgen,
            luma_pyramid,
            presence,
            resample,
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
            meta_bins,
            #[cfg(feature = "native")]
            sensor_stage: SensorStagePasses::new(ctx, &registry),
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
            depth16: std::sync::OnceLock::new(),
        }
    }

    pub fn depth16(&self, ctx: &Arc<GpuContext>) -> &Depth16Passes {
        self.depth16.get_or_init(|| {
            let depth = DisplayDepth::Sixteen;
            Depth16Passes {
                process_fast: ProcessFastPass::new_with_mask(
                    ctx,
                    &self.registry,
                    StageMask::fast(),
                    depth,
                    "process-fast16",
                ),
                process_post_wb: ProcessFastPass::new_with_mask(
                    ctx,
                    &self.registry,
                    StageMask::tone_color(),
                    depth,
                    "process-post16",
                ),
                effects_tone: EffectsTonePass::new(ctx, depth),
                lut: LutPass::new(ctx, depth),
                dcp_look: DcpHueSatPass::new_look(ctx, depth.format()),
                meta_bins: MetaBinsPasses::new(ctx, depth),
            }
        })
    }
}
