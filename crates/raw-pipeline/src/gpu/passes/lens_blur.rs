// color-space: linear scene-referred Rgba16Float in/out, Rgba32Float grid scratch
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureFormat, TextureViewDimension};

use crate::gpu::context::GpuContext;
use crate::gpu::shader_builder::LENS_WARP_WGSL;
use crate::ops::lens_blur::coc::COC_LEVELS;
use crate::ops::lens_blur::kernel::{
    APERTURE_ROTATION, HIGHLIGHT_GAIN, HIGHLIGHT_HI, HIGHLIGHT_LO, UPSAMPLE_SHARPNESS,
};
use crate::ops::wgsl::f32_lit;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry_with, uniform_entry};

pub const GRID_FORMAT: TextureFormat = TextureFormat::Rgba32Float;
pub const DEPTH_FORMAT: TextureFormat = TextureFormat::R8Unorm;
pub const TABLE_VEC4S: usize = COC_LEVELS / 4;
pub const UNIFORM_SIZE: u64 = size_of::<LensBlurParams>() as u64;

const COMMON_WGSL: &str = include_str!("../../../assets/shaders/lens_blur_common.wgsl");
const DEPTH_WGSL: &str = include_str!("../../../assets/shaders/lens_blur_depth.wgsl");
const DOWN_WGSL: &str = include_str!("../../../assets/shaders/lens_blur_down.wgsl");
const GATHER_WGSL: &str = include_str!("../../../assets/shaders/lens_blur_gather.wgsl");
const APPLY_WGSL: &str = include_str!("../../../assets/shaders/lens_blur_apply.wgsl");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LensBlurParams {
    pub size: [u32; 2],
    pub origin: [u32; 2],
    pub full: [u32; 2],
    pub grid_origin: [u32; 2],
    pub grid_size: [u32; 2],
    pub ds: u32,
    pub reach: i32,
    pub orient: [u32; 3],
    pub _pad0: u32,
    pub lens: [f32; 4],
    pub radius: f32,
    pub boost: f32,
    pub blades: u32,
    pub _pad: u32,
    pub table: [[f32; 4]; TABLE_VEC4S],
}

fn constants() -> String {
    let mut consts = format!("const COC_LEVELS: u32 = {COC_LEVELS}u;\n");
    for (name, value) in [
        ("PI", std::f32::consts::PI),
        ("TAU", std::f32::consts::TAU),
        ("APERTURE_ROTATION", APERTURE_ROTATION),
        ("HIGHLIGHT_GAIN", HIGHLIGHT_GAIN),
        ("HIGHLIGHT_LO", HIGHLIGHT_LO),
        ("HIGHLIGHT_HI", HIGHLIGHT_HI),
        ("UPSAMPLE_SHARPNESS", UPSAMPLE_SHARPNESS),
    ] {
        consts.push_str(&format!("const {name}: f32 = {};\n", f32_lit(value)));
    }
    consts
}

pub fn down_wgsl() -> String {
    [
        constants().as_str(),
        COMMON_WGSL,
        LENS_WARP_WGSL,
        DEPTH_WGSL,
        DOWN_WGSL,
    ]
    .concat()
}

pub fn gather_wgsl() -> String {
    [constants().as_str(), COMMON_WGSL, GATHER_WGSL].concat()
}

pub fn apply_wgsl() -> String {
    [
        constants().as_str(),
        COMMON_WGSL,
        LENS_WARP_WGSL,
        DEPTH_WGSL,
        APPLY_WGSL,
    ]
    .concat()
}

pub struct LensBlurPasses {
    pub down_layout: BindGroupLayout,
    pub down_pipeline: ComputePipeline,
    pub gather_layout: BindGroupLayout,
    pub gather_pipeline: ComputePipeline,
    pub apply_layout: BindGroupLayout,
    pub apply_pipeline: ComputePipeline,
}

fn texture(binding: u32) -> wgpu::BindGroupLayoutEntry {
    tex_entry_with(binding, false, TextureViewDimension::D2)
}

impl LensBlurPasses {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let down_layout = make_layout(
            ctx,
            "lens-blur-down-bgl",
            &[
                uniform_entry(0, UNIFORM_SIZE),
                texture(1),
                texture(2),
                storage_entry(3, GRID_FORMAT),
            ],
        );
        let gather_layout = make_layout(
            ctx,
            "lens-blur-gather-bgl",
            &[
                uniform_entry(0, UNIFORM_SIZE),
                texture(1),
                storage_entry(2, GRID_FORMAT),
            ],
        );
        let apply_layout = make_layout(
            ctx,
            "lens-blur-apply-bgl",
            &[
                uniform_entry(0, UNIFORM_SIZE),
                texture(1),
                texture(2),
                texture(3),
                texture(4),
                storage_entry(5, ctx.linear_format),
            ],
        );
        Self {
            down_pipeline: make_pipeline(ctx, &down_layout, "lens_blur_down.wgsl", &down_wgsl()),
            gather_pipeline: make_pipeline(
                ctx,
                &gather_layout,
                "lens_blur_gather.wgsl",
                &gather_wgsl(),
            ),
            apply_pipeline: make_pipeline(
                ctx,
                &apply_layout,
                "lens_blur_apply.wgsl",
                &apply_wgsl(),
            ),
            down_layout,
            gather_layout,
            apply_layout,
        }
    }
}
