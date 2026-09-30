// color-space: linear scene-referred Rgba16Float in/out, R32Float luma and Rgba32Float Y/Pb/Pr scratch
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, BindGroupLayoutEntry, ComputePipeline, TextureFormat, TextureViewDimension,
};

use crate::gpu::context::GpuContext;
use crate::ops::denoise::atrous::B3;
use crate::ops::denoise::estimate::{
    HIST_LEN, LUM_BINS, MAG_BINS, MAG_BINS_PER_OCTAVE, MAG_LOG2_MIN, SAMPLE_STRIDE,
};
use crate::ops::denoise::{PB_DEN, PR_DEN};
use crate::tone::shared::{LUMA_B, LUMA_G, LUMA_R};

use super::common::{
    make_layout, make_pipeline_raw, storage_buffer_entry_rw, storage_entry, tex_entry_with,
    uniform_entry,
};
use super::demosaic::linear_format_str;

pub const NR_LUMA_FORMAT: TextureFormat = TextureFormat::R32Float;
pub const NR_CHROMA_FORMAT: TextureFormat = TextureFormat::Rgba32Float;
pub const NR_HIST_TILE: u32 = 64;
pub const NR_INIT_READ: u32 = 0;
pub const NR_INIT_ZERO: u32 = 1;
pub const NR_INIT_CARRY_REFERENCE: u32 = 2;

const COMMON_WGSL: &str = include_str!("../../../assets/shaders/nr_common.wgsl");
pub const NR_LUMA_INIT_WGSL: &str = include_str!("../../../assets/shaders/nr_luma_init.wgsl");
pub const NR_ATROUS_WGSL: &str = include_str!("../../../assets/shaders/nr_atrous.wgsl");
pub const NR_HIST_WGSL: &str = include_str!("../../../assets/shaders/nr_hist.wgsl");
pub const NR_SHRINK_WGSL: &str = include_str!("../../../assets/shaders/nr_shrink.wgsl");
pub const NR_LUMA_FINISH_WGSL: &str = include_str!("../../../assets/shaders/nr_luma_finish.wgsl");
pub const NR_CHROMA_DOWN_WGSL: &str = include_str!("../../../assets/shaders/nr_chroma_down.wgsl");
pub const NR_CHROMA_APPLY_WGSL: &str = include_str!("../../../assets/shaders/nr_chroma_apply.wgsl");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NrSizeParams {
    pub size: [u32; 2],
    pub _pad: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NrAtrousParams {
    pub size: [u32; 2],
    pub step: u32,
    pub axis: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NrHistParams {
    pub size: [u32; 2],
    pub lo_size: [u32; 2],
    pub lo: u32,
    pub count: u32,
    pub offset: u32,
    pub fine: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NrShrinkParams {
    pub curves: [f32; 4],
    pub size: [u32; 2],
    pub step: u32,
    pub lo: u32,
    pub count: u32,
    pub init: u32,
    pub lambda: f32,
    pub mu: f32,
    pub keep: f32,
    pub _pad: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NrApplyParams {
    pub curves: [f32; 4],
    pub size: [u32; 2],
    pub lo_size: [u32; 2],
    pub lambda: f32,
    pub mu: f32,
    pub keep: f32,
    pub _pad: u32,
}

pub fn nr_wgsl(body: &str) -> String {
    format!(
        "const LUMA_R: f32 = {LUMA_R:?};\n\
         const LUMA_G: f32 = {LUMA_G:?};\n\
         const LUMA_B: f32 = {LUMA_B:?};\n\
         const PB_DEN: f32 = {PB_DEN:?};\n\
         const PR_DEN: f32 = {PR_DEN:?};\n\
         const B3_0: f32 = {:?};\n\
         const B3_1: f32 = {:?};\n\
         const B3_2: f32 = {:?};\n\
         const LUM_BINS: u32 = {LUM_BINS}u;\n\
         const MAG_BINS: u32 = {MAG_BINS}u;\n\
         const MAG_LOG2_MIN: f32 = {MAG_LOG2_MIN:?};\n\
         const MAG_BINS_PER_OCTAVE: f32 = {MAG_BINS_PER_OCTAVE:?};\n\
         const HIST_LEN: u32 = {HIST_LEN}u;\n\
         const HIST_TILE: u32 = {NR_HIST_TILE}u;\n\
         const SAMPLE_STRIDE: u32 = {SAMPLE_STRIDE}u;\n\
         const INIT_ZERO: u32 = {NR_INIT_ZERO}u;\n\
         const INIT_CARRY_REFERENCE: u32 = {NR_INIT_CARRY_REFERENCE}u;\n\
         {COMMON_WGSL}\n{body}",
        B3[0], B3[1], B3[2],
    )
}

pub struct NrKernel {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl NrKernel {
    fn new(
        ctx: &Arc<GpuContext>,
        label: &str,
        entries: &[BindGroupLayoutEntry],
        body: &str,
        storage: TextureFormat,
    ) -> Self {
        let layout = make_layout(ctx, label, entries);
        let format = match storage {
            TextureFormat::R32Float => "r32float",
            other => linear_format_str(other),
        };
        let src = nr_wgsl(body).replace("rgba16float", format);
        let pipeline = make_pipeline_raw(ctx, &layout, label, &src);
        Self { layout, pipeline }
    }
}

pub struct NrPlaneKernels {
    pub atrous: NrKernel,
    pub shrink: NrKernel,
}

impl NrPlaneKernels {
    fn new(ctx: &Arc<GpuContext>, format: TextureFormat) -> Self {
        Self {
            atrous: NrKernel::new(
                ctx,
                "nr_atrous.wgsl",
                &[
                    uniform_entry(0, size_of::<NrAtrousParams>() as u64),
                    read(1),
                    storage_entry(2, format),
                ],
                NR_ATROUS_WGSL,
                format,
            ),
            shrink: NrKernel::new(
                ctx,
                "nr_shrink.wgsl",
                &[
                    uniform_entry(0, size_of::<NrShrinkParams>() as u64),
                    read(1),
                    read(2),
                    read(3),
                    storage_entry(4, format),
                ],
                NR_SHRINK_WGSL,
                format,
            ),
        }
    }
}

pub struct NrPasses {
    pub luma: NrPlaneKernels,
    pub chroma: NrPlaneKernels,
    pub hist: NrKernel,
    pub luma_init: NrKernel,
    pub luma_finish: NrKernel,
    pub chroma_down: NrKernel,
    pub chroma_apply: NrKernel,
}

fn read(binding: u32) -> BindGroupLayoutEntry {
    tex_entry_with(binding, false, TextureViewDimension::D2)
}

impl NrPasses {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let linear = ctx.linear_format;
        let size_uniform = uniform_entry(0, size_of::<NrSizeParams>() as u64);
        Self {
            luma: NrPlaneKernels::new(ctx, NR_LUMA_FORMAT),
            chroma: NrPlaneKernels::new(ctx, NR_CHROMA_FORMAT),
            hist: NrKernel::new(
                ctx,
                "nr_hist.wgsl",
                &[
                    uniform_entry(0, size_of::<NrHistParams>() as u64),
                    read(1),
                    read(2),
                    read(3),
                    storage_buffer_entry_rw(4),
                ],
                NR_HIST_WGSL,
                linear,
            ),
            luma_init: NrKernel::new(
                ctx,
                "nr_luma_init.wgsl",
                &[size_uniform, read(1), storage_entry(2, NR_LUMA_FORMAT)],
                NR_LUMA_INIT_WGSL,
                NR_LUMA_FORMAT,
            ),
            luma_finish: NrKernel::new(
                ctx,
                "nr_luma_finish.wgsl",
                &[size_uniform, read(1), read(2), storage_entry(3, linear)],
                NR_LUMA_FINISH_WGSL,
                linear,
            ),
            chroma_down: NrKernel::new(
                ctx,
                "nr_chroma_down.wgsl",
                &[
                    size_uniform,
                    read(1),
                    storage_entry(2, NR_CHROMA_FORMAT),
                    storage_entry(3, NR_CHROMA_FORMAT),
                ],
                NR_CHROMA_DOWN_WGSL,
                NR_CHROMA_FORMAT,
            ),
            chroma_apply: NrKernel::new(
                ctx,
                "nr_chroma_apply.wgsl",
                &[
                    uniform_entry(0, size_of::<NrApplyParams>() as u64),
                    read(1),
                    read(2),
                    read(3),
                    storage_entry(4, linear),
                ],
                NR_CHROMA_APPLY_WGSL,
                linear,
            ),
        }
    }
}
