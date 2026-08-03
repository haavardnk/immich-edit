use std::borrow::Cow;
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferSize,
    ComputePipeline, ComputePipelineDescriptor, PipelineLayoutDescriptor, SamplerBindingType,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StorageTextureAccess, TextureFormat,
    TextureSampleType, TextureViewDimension,
};

use crate::gpu::context::GpuContext;

use super::demosaic::linear_format_str;

pub(super) fn uniform_entry(binding: u32, size: u64) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: BufferSize::new(size),
        },
        count: None,
    }
}

pub(super) fn uniform_entry_unsized(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub(super) fn storage_buffer_entry(binding: u32) -> BindGroupLayoutEntry {
    storage_buffer_entry_with(binding, true)
}

pub(super) fn storage_buffer_entry_rw(binding: u32) -> BindGroupLayoutEntry {
    storage_buffer_entry_with(binding, false)
}

fn storage_buffer_entry_with(binding: u32, read_only: bool) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub(super) fn tex_entry(binding: u32) -> BindGroupLayoutEntry {
    tex_entry_with(binding, true, TextureViewDimension::D2)
}

pub(super) fn tex_entry_with(
    binding: u32,
    filterable: bool,
    view_dimension: TextureViewDimension,
) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable },
            view_dimension,
            multisampled: false,
        },
        count: None,
    }
}

pub(super) fn storage_entry(binding: u32, format: TextureFormat) -> BindGroupLayoutEntry {
    storage_entry_with(binding, format, StorageTextureAccess::WriteOnly)
}

pub(super) fn storage_entry_with(
    binding: u32,
    format: TextureFormat,
    access: StorageTextureAccess,
) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::StorageTexture {
            access,
            format,
            view_dimension: TextureViewDimension::D2,
        },
        count: None,
    }
}

pub(super) fn sampler_entry(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Sampler(SamplerBindingType::Filtering),
        count: None,
    }
}

pub(super) fn make_layout(
    ctx: &Arc<GpuContext>,
    label: &str,
    entries: &[BindGroupLayoutEntry],
) -> BindGroupLayout {
    ctx.device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(label),
            entries,
        })
}

pub(super) fn make_pipeline(
    ctx: &Arc<GpuContext>,
    layout: &BindGroupLayout,
    label: &str,
    wgsl: &str,
) -> ComputePipeline {
    let src = wgsl.replace("rgba16float", linear_format_str(ctx.linear_format));
    make_pipeline_raw(ctx, layout, label, &src)
}

pub(super) fn make_pipeline_raw(
    ctx: &Arc<GpuContext>,
    layout: &BindGroupLayout,
    label: &str,
    wgsl: &str,
) -> ComputePipeline {
    let device = &ctx.device;
    let module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some(label),
        source: ShaderSource::Wgsl(Cow::Borrowed(wgsl)),
    });
    let pl = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&pl),
        module: &module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    })
}
