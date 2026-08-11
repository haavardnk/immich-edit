// color-space: linear scene-referred Rgba16Float in/out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, Sampler, SamplerDescriptor, TextureFormat};

use crate::gpu::context::GpuContext;

use super::common::{
    make_layout, make_pipeline, sampler_entry, storage_entry, tex_entry, tex_entry_with,
    uniform_entry,
};

pub const MOMENT_FORMAT: TextureFormat = TextureFormat::Rgba32Float;

pub const DOWNSAMPLE_UNIFORM_SIZE: u64 = 16;
pub const NORM_UNIFORM_SIZE: u64 = 32;
pub const MIN_UNIFORM_SIZE: u64 = 16;
pub const PACK_UNIFORM_SIZE: u64 = 16;
pub const BOX_UNIFORM_SIZE: u64 = 16;
pub const AB_UNIFORM_SIZE: u64 = 16;
pub const APPLY_UNIFORM_SIZE: u64 = 48;

fn make_layout_3(
    ctx: &Arc<GpuContext>,
    label: &str,
    uniform_size: u64,
    src_filterable: bool,
) -> BindGroupLayout {
    make_layout(
        ctx,
        label,
        &[
            uniform_entry(0, uniform_size),
            tex_entry_with(1, src_filterable, wgpu::TextureViewDimension::D2),
            storage_entry(2, MOMENT_FORMAT),
        ],
    )
}

fn make_layout_4(ctx: &Arc<GpuContext>, label: &str, uniform_size: u64) -> BindGroupLayout {
    make_layout(
        ctx,
        label,
        &[
            uniform_entry(0, uniform_size),
            tex_entry(1),
            tex_entry_with(2, false, wgpu::TextureViewDimension::D2),
            storage_entry(3, MOMENT_FORMAT),
        ],
    )
}

fn make_layout_downsample(ctx: &Arc<GpuContext>, label: &str) -> BindGroupLayout {
    make_layout(
        ctx,
        label,
        &[
            uniform_entry(0, DOWNSAMPLE_UNIFORM_SIZE),
            tex_entry(1),
            sampler_entry(2),
            storage_entry(3, ctx.linear_format),
        ],
    )
}

fn make_layout_apply(ctx: &Arc<GpuContext>, label: &str) -> BindGroupLayout {
    make_layout(
        ctx,
        label,
        &[
            uniform_entry(0, APPLY_UNIFORM_SIZE),
            tex_entry(1),
            tex_entry_with(2, false, wgpu::TextureViewDimension::D2),
            storage_entry(3, ctx.linear_format),
        ],
    )
}

pub struct DehazePasses {
    pub downsample_layout: BindGroupLayout,
    pub downsample_pipeline: ComputePipeline,
    pub norm_layout: BindGroupLayout,
    pub norm_pipeline: ComputePipeline,
    pub min_layout: BindGroupLayout,
    pub min_pipeline: ComputePipeline,
    pub pack_layout: BindGroupLayout,
    pub pack_pipeline: ComputePipeline,
    pub box_layout: BindGroupLayout,
    pub box_pipeline: ComputePipeline,
    pub ab_layout: BindGroupLayout,
    pub ab_pipeline: ComputePipeline,
    pub apply_layout: BindGroupLayout,
    pub apply_pipeline: ComputePipeline,
    pub linear_sampler: Sampler,
}

impl DehazePasses {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let downsample_layout = make_layout_downsample(ctx, "dehaze-downsample-bgl");
        let downsample_pipeline = make_pipeline(
            ctx,
            &downsample_layout,
            "dehaze_downsample.wgsl",
            include_str!("../../../assets/shaders/dehaze_downsample.wgsl"),
        );

        let norm_layout = make_layout_3(ctx, "dehaze-norm-bgl", NORM_UNIFORM_SIZE, true);
        let norm_pipeline = make_pipeline(
            ctx,
            &norm_layout,
            "dehaze_norm.wgsl",
            include_str!("../../../assets/shaders/dehaze_norm.wgsl"),
        );

        let min_layout = make_layout_3(ctx, "dehaze-min-bgl", MIN_UNIFORM_SIZE, false);
        let min_pipeline = make_pipeline(
            ctx,
            &min_layout,
            "dehaze_min.wgsl",
            include_str!("../../../assets/shaders/dehaze_min.wgsl"),
        );

        let pack_layout = make_layout_4(ctx, "dehaze-pack-bgl", PACK_UNIFORM_SIZE);
        let pack_pipeline = make_pipeline(
            ctx,
            &pack_layout,
            "dehaze_pack.wgsl",
            include_str!("../../../assets/shaders/dehaze_pack.wgsl"),
        );

        let box_layout = make_layout_3(ctx, "dehaze-box-bgl", BOX_UNIFORM_SIZE, false);
        let box_pipeline = make_pipeline(
            ctx,
            &box_layout,
            "dehaze_box.wgsl",
            include_str!("../../../assets/shaders/dehaze_box.wgsl"),
        );

        let ab_layout = make_layout_3(ctx, "dehaze-ab-bgl", AB_UNIFORM_SIZE, false);
        let ab_pipeline = make_pipeline(
            ctx,
            &ab_layout,
            "dehaze_ab.wgsl",
            include_str!("../../../assets/shaders/dehaze_ab.wgsl"),
        );

        let apply_layout = make_layout_apply(ctx, "dehaze-apply-bgl");
        let apply_pipeline = make_pipeline(
            ctx,
            &apply_layout,
            "dehaze_apply.wgsl",
            include_str!("../../../assets/shaders/dehaze_apply.wgsl"),
        );

        let linear_sampler = ctx.device.create_sampler(&SamplerDescriptor {
            label: Some("dehaze-linear-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            downsample_layout,
            downsample_pipeline,
            norm_layout,
            norm_pipeline,
            min_layout,
            min_pipeline,
            pack_layout,
            pack_pipeline,
            box_layout,
            box_pipeline,
            ab_layout,
            ab_pipeline,
            apply_layout,
            apply_pipeline,
            linear_sampler,
        }
    }
}
