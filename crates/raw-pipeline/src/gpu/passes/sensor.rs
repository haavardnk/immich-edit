// color-space: linear scene-referred Rgba16Float in/out
use std::borrow::Cow;
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, ComputePipeline, ComputePipelineDescriptor, PipelineLayoutDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StorageTextureAccess, TextureSampleType,
    TextureViewDimension,
};

use crate::gpu::context::GpuContext;
use crate::gpu::passes::demosaic::linear_format_str;

pub struct SensorPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl SensorPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let device = &ctx.device;
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("sensor-bgl"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: ctx.linear_format,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let src = include_str!("../../../assets/shaders/sensor.wgsl")
            .replace("rgba16float", linear_format_str(ctx.linear_format));
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("sensor.wgsl"),
            source: ShaderSource::Wgsl(Cow::Owned(src)),
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("sensor-pl"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("sensor-cp"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });
        Self { layout, pipeline }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SensorParams {
    pub size: [u32; 2],
    pub zoom: f32,
    pub _pad0: f32,
    pub coeffs: [f32; 4],
    pub ca_vig: [f32; 4],
}

impl SensorParams {
    pub fn from_edits(lens: &crate::edits::LensEdits, w: u32, h: u32) -> Self {
        let (k1, k2, k3) = crate::ops::lens_distortion::distortion_coeffs(lens);
        let (red_scale, blue_scale) = crate::ops::lens_ca::ca_scales(lens);
        let (vk1, vk2, vk3) = crate::ops::lens_vignette::vignette_coeffs(lens);
        let zoom = crate::ops::lens_distortion::distortion_zoom(lens);
        Self {
            size: [w, h],
            zoom,
            _pad0: 0.0,
            coeffs: [k1, k2, k3, vk1],
            ca_vig: [red_scale, blue_scale, vk2, vk3],
        }
    }
}
