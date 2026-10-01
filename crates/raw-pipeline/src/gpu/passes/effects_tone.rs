// color-space: linear scene-referred Rgba16Float in → sRGB-encoded Rgba8Unorm + linear Rgba16Float out (vignette + grain in linear, then tone-map)
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;
use crate::gpu::display_depth::{DISPLAY_STORE_INJECT, DisplayDepth};
use crate::ops::vignette::{
    VIGNETTE_DARKEN_STOPS, VIGNETTE_HIGHLIGHT_HI, VIGNETTE_HIGHLIGHT_LO,
    VIGNETTE_HIGHLIGHT_PRIORITY,
};
use crate::ops::wgsl::f32_lit;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry, uniform_entry};

pub const EFFECTS_TONE_UNIFORM_SIZE: u64 = size_of::<EffectsToneParams>() as u64;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EffectsToneParams {
    pub size: [u32; 2],
    pub _pad0: [u32; 2],
    pub vignette: [f32; 4],
    pub grain: [f32; 3],
    pub display_p3: u32,
    pub warn_flags: u32,
    pub output_scale: f32,
    pub _pad1: [u32; 2],
    pub roi: [f32; 4],
}

pub fn effects_tone_wgsl(depth: DisplayDepth) -> String {
    let mut consts = String::new();
    for (name, value) in [
        ("VIGNETTE_DARKEN_STOPS", VIGNETTE_DARKEN_STOPS),
        ("VIGNETTE_HIGHLIGHT_PRIORITY", VIGNETTE_HIGHLIGHT_PRIORITY),
        ("VIGNETTE_HIGHLIGHT_LO", VIGNETTE_HIGHLIGHT_LO),
        ("VIGNETTE_HIGHLIGHT_HI", VIGNETTE_HIGHLIGHT_HI),
    ] {
        consts.push_str(&format!("const {name}: f32 = {};\n", f32_lit(value)));
    }
    include_str!("../../../assets/shaders/effects_tone.wgsl")
        .replace(DISPLAY_STORE_INJECT, &depth.store_wgsl(2))
        .replace("// VIGNETTE_CONST_INJECT", &consts)
        .replace("// TONE_WGSL_INJECT", crate::tone::wgsl::tone_wgsl())
}

pub struct EffectsTonePass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl EffectsTonePass {
    pub fn new(ctx: &Arc<GpuContext>, depth: DisplayDepth) -> Self {
        let layout = make_layout(
            ctx,
            "effects-tone-bgl",
            &[
                uniform_entry(0, EFFECTS_TONE_UNIFORM_SIZE),
                tex_entry(1),
                storage_entry(2, depth.format()),
                storage_entry(3, ctx.linear_format),
            ],
        );
        let pipeline = make_pipeline(ctx, &layout, "effects_tone.wgsl", &effects_tone_wgsl(depth));

        Self { layout, pipeline }
    }
}
