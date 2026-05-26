// color-space: linear scene-referred Rgba16Float in/out
use std::borrow::Cow;
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferSize,
    ComputePipeline, ComputePipelineDescriptor, PipelineLayoutDescriptor, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    StorageTextureAccess, TextureSampleType, TextureViewDimension,
};

use crate::gpu::context::GpuContext;

use super::demosaic::linear_format_str;

pub const DOWNSAMPLE_UNIFORM_SIZE: u64 = 16;
pub const NORM_UNIFORM_SIZE: u64 = 32;
pub const MIN_UNIFORM_SIZE: u64 = 16;
pub const PACK_UNIFORM_SIZE: u64 = 16;
pub const BOX_UNIFORM_SIZE: u64 = 16;
pub const AB_UNIFORM_SIZE: u64 = 16;
pub const APPLY_UNIFORM_SIZE: u64 = 48;

fn make_pipeline(
    ctx: &Arc<GpuContext>,
    layout: &BindGroupLayout,
    label: &str,
    wgsl: &str,
) -> ComputePipeline {
    let device = &ctx.device;
    let src = wgsl.replace("rgba16float", linear_format_str(ctx.linear_format));
    let module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some(label),
        source: ShaderSource::Wgsl(Cow::Owned(src)),
    });
    let pl = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[layout],
        push_constant_ranges: &[],
    });
    device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&pl),
        module: &module,
        entry_point: "main",
        compilation_options: Default::default(),
        cache: None,
    })
}

fn uniform_entry(size: u64) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: BufferSize::new(size),
        },
        count: None,
    }
}

fn tex_entry(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn storage_entry(binding: u32, format: wgpu::TextureFormat) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::StorageTexture {
            access: StorageTextureAccess::WriteOnly,
            format,
            view_dimension: TextureViewDimension::D2,
        },
        count: None,
    }
}

fn make_layout_3(ctx: &Arc<GpuContext>, label: &str, uniform_size: u64) -> BindGroupLayout {
    ctx.device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(label),
            entries: &[
                uniform_entry(uniform_size),
                tex_entry(1),
                storage_entry(2, ctx.linear_format),
            ],
        })
}

fn make_layout_4(ctx: &Arc<GpuContext>, label: &str, uniform_size: u64) -> BindGroupLayout {
    ctx.device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(label),
            entries: &[
                uniform_entry(uniform_size),
                tex_entry(1),
                tex_entry(2),
                storage_entry(3, ctx.linear_format),
            ],
        })
}

fn sampler_entry(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Sampler(SamplerBindingType::Filtering),
        count: None,
    }
}

fn make_layout_downsample(ctx: &Arc<GpuContext>, label: &str) -> BindGroupLayout {
    ctx.device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(label),
            entries: &[
                uniform_entry(DOWNSAMPLE_UNIFORM_SIZE),
                tex_entry(1),
                sampler_entry(2),
                storage_entry(3, ctx.linear_format),
            ],
        })
}

fn make_layout_apply(ctx: &Arc<GpuContext>, label: &str) -> BindGroupLayout {
    ctx.device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(label),
            entries: &[
                uniform_entry(APPLY_UNIFORM_SIZE),
                tex_entry(1),
                tex_entry(2),
                sampler_entry(3),
                storage_entry(4, ctx.linear_format),
            ],
        })
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

        let norm_layout = make_layout_3(ctx, "dehaze-norm-bgl", NORM_UNIFORM_SIZE);
        let norm_pipeline = make_pipeline(
            ctx,
            &norm_layout,
            "dehaze_norm.wgsl",
            include_str!("../../../assets/shaders/dehaze_norm.wgsl"),
        );

        let min_layout = make_layout_3(ctx, "dehaze-min-bgl", MIN_UNIFORM_SIZE);
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

        let box_layout = make_layout_3(ctx, "dehaze-box-bgl", BOX_UNIFORM_SIZE);
        let box_pipeline = make_pipeline(
            ctx,
            &box_layout,
            "dehaze_box.wgsl",
            include_str!("../../../assets/shaders/dehaze_box.wgsl"),
        );

        let ab_layout = make_layout_3(ctx, "dehaze-ab-bgl", AB_UNIFORM_SIZE);
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
            mipmap_filter: wgpu::FilterMode::Nearest,
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
