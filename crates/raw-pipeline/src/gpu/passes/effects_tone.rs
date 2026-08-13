// color-space: linear scene-referred Rgba16Float in → sRGB-encoded Rgba8Unorm + linear Rgba16Float out (vignette + grain in linear, then tone-map)
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry, uniform_entry};

pub const EFFECTS_TONE_UNIFORM_SIZE: u64 = size_of::<EffectsToneParams>() as u64;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EffectsToneParams {
    pub size: [u32; 2],
    pub _pad0: [u32; 2],
    pub vignette: [f32; 4],
    pub grain: [f32; 3],
    pub _pad1: [f32; 3],
    pub display_p3: u32,
    pub warn_flags: u32,
    pub roi: [f32; 4],
}

pub struct EffectsTonePass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl EffectsTonePass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "effects-tone-bgl",
            &[
                uniform_entry(0, EFFECTS_TONE_UNIFORM_SIZE),
                tex_entry(1),
                storage_entry(2, wgpu::TextureFormat::Rgba8Unorm),
                storage_entry(3, ctx.linear_format),
            ],
        );
        let src = include_str!("../../../assets/shaders/effects_tone.wgsl")
            .replace("// TONE_WGSL_INJECT", crate::tone::wgsl::tone_wgsl());
        let pipeline = make_pipeline(ctx, &layout, "effects_tone.wgsl", &src);

        Self { layout, pipeline }
    }
}
