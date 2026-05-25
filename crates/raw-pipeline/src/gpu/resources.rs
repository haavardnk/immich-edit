use wgpu::{
    Buffer, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use super::context::GpuContext;
use super::helpers::round_up_256;
use super::readback::{make_readback_buffer, make_readback_buffer_f16};

pub(super) struct OutputTargets {
    pub texture: Texture,
    pub readback: Buffer,
    pub linear_texture: Texture,
    pub linear_readback: Buffer,
    pub mask_accum_alt: Texture,
    pub mask_scratch_linear: Texture,
    pub mask_scratch_tone: Texture,
    pub mask_weight: Texture,
    pub alloc_w: u32,
    pub alloc_h: u32,
}

impl OutputTargets {
    pub fn fits(&self, w: u32, h: u32) -> bool {
        self.alloc_w >= w && self.alloc_h >= h
    }

    pub fn allocate(ctx: &GpuContext, out_w: u32, out_h: u32) -> Self {
        let device = &ctx.device;
        let need_w = round_up_256(out_w);
        let need_h = round_up_256(out_h);
        let linear_extra_usage = TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
        let make_linear = |label: &'static str, usage: TextureUsages| -> Texture {
            device.create_texture(&TextureDescriptor {
                label: Some(label),
                size: Extent3d {
                    width: need_w,
                    height: need_h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage,
                view_formats: &[],
            })
        };
        Self {
            texture: device.create_texture(&TextureDescriptor {
                label: Some("output"),
                size: Extent3d {
                    width: need_w,
                    height: need_h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
                view_formats: &[],
            }),
            readback: make_readback_buffer(device, need_w, need_h),
            linear_texture: device.create_texture(&TextureDescriptor {
                label: Some("linear-output"),
                size: Extent3d {
                    width: need_w,
                    height: need_h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST,
                view_formats: &[],
            }),
            linear_readback: make_readback_buffer_f16(device, need_w, need_h),
            mask_accum_alt: make_linear(
                "mask-accum-alt",
                linear_extra_usage | TextureUsages::COPY_SRC,
            ),
            mask_scratch_linear: make_linear("mask-scratch-linear", linear_extra_usage),
            mask_scratch_tone: device.create_texture(&TextureDescriptor {
                label: Some("mask-scratch-tone"),
                size: Extent3d {
                    width: need_w,
                    height: need_h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            }),
            mask_weight: device.create_texture(&TextureDescriptor {
                label: Some("mask-weight"),
                size: Extent3d {
                    width: need_w,
                    height: need_h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R32Float,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }),
            alloc_w: need_w,
            alloc_h: need_h,
        }
    }
}

pub(super) struct SharpenTargets {
    pub blur_h: Texture,
    pub blur_full: Texture,
    pub post_lin: Texture,
    pub alloc_w: u32,
    pub alloc_h: u32,
}

impl SharpenTargets {
    pub fn fits(&self, w: u32, h: u32) -> bool {
        self.alloc_w >= w && self.alloc_h >= h
    }

    pub fn allocate(ctx: &GpuContext, out_w: u32, out_h: u32) -> Self {
        let device = &ctx.device;
        let need_w = round_up_256(out_w);
        let need_h = round_up_256(out_h);
        let make = |label: &'static str, usage: TextureUsages| -> Texture {
            device.create_texture(&TextureDescriptor {
                label: Some(label),
                size: Extent3d {
                    width: need_w,
                    height: need_h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: ctx.linear_format,
                usage,
                view_formats: &[],
            })
        };
        let base = TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
        Self {
            blur_h: make("sharpen-blur-h", base),
            blur_full: make("sharpen-blur-full", base),
            post_lin: make("output-post-lin", base | TextureUsages::COPY_SRC),
            alloc_w: need_w,
            alloc_h: need_h,
        }
    }
}
