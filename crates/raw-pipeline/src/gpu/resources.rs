use wgpu::{Buffer, BufferDescriptor, BufferUsages, Texture, TextureFormat, TextureUsages};

use super::context::GpuContext;
use super::helpers::round_up_256;
use super::passes::meta_bins::HISTOGRAM_COUNTS;
use super::readback::make_readback_buffer;
use super::texture::{STORAGE_SAMPLED, texture_2d};
use crate::scopes::SCOPE_CELLS;

pub(super) const HISTOGRAM_BYTES: u64 = (HISTOGRAM_COUNTS * size_of::<u32>()) as u64;
pub(super) const SCOPE_BYTES: u64 = (SCOPE_CELLS * size_of::<u32>()) as u64;

pub(super) struct OutputTargets {
    pub texture: Texture,
    pub readback: Buffer,
    pub linear_texture: Texture,
    pub histogram_counts: Buffer,
    pub scope_counts: Buffer,
    pub meta_readback: Buffer,
    pub mask_accum_alt: Texture,
    pub mask_selector: Texture,
    pub mask_scratch_linear: Texture,
    pub mask_scratch_tone: Texture,
    pub mask_weight: Texture,
    pub mask_sharpen: Texture,
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
        let make = |label: &str, format: TextureFormat, usage: TextureUsages| -> Texture {
            texture_2d(device, label, format, (need_w, need_h), 1, usage)
        };
        let copy_both = TextureUsages::COPY_SRC | TextureUsages::COPY_DST;
        Self {
            texture: make(
                "output",
                TextureFormat::Rgba8Unorm,
                STORAGE_SAMPLED | copy_both,
            ),
            readback: make_readback_buffer(device, need_w, need_h),
            linear_texture: make(
                "linear-output",
                TextureFormat::Rgba16Float,
                STORAGE_SAMPLED | copy_both,
            ),
            histogram_counts: make_counts_buffer(device, "histogram-counts", HISTOGRAM_BYTES),
            scope_counts: make_counts_buffer(device, "scope-counts", SCOPE_BYTES),
            meta_readback: device.create_buffer(&BufferDescriptor {
                label: Some("meta-readback"),
                size: HISTOGRAM_BYTES + SCOPE_BYTES,
                usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            mask_accum_alt: make(
                "mask-accum-alt",
                TextureFormat::Rgba16Float,
                STORAGE_SAMPLED | TextureUsages::COPY_SRC,
            ),
            mask_selector: make("mask-selector", TextureFormat::Rgba32Float, STORAGE_SAMPLED),
            mask_scratch_linear: make(
                "mask-scratch-linear",
                TextureFormat::Rgba16Float,
                STORAGE_SAMPLED,
            ),
            mask_scratch_tone: make(
                "mask-scratch-tone",
                TextureFormat::Rgba8Unorm,
                STORAGE_SAMPLED | TextureUsages::COPY_SRC,
            ),
            mask_weight: make("mask-weight", TextureFormat::R32Float, STORAGE_SAMPLED),
            mask_sharpen: make(
                "mask-sharpen",
                TextureFormat::R32Float,
                STORAGE_SAMPLED | TextureUsages::COPY_DST,
            ),
            alloc_w: need_w,
            alloc_h: need_h,
        }
    }
}

fn make_counts_buffer(device: &wgpu::Device, label: &'static str, size: u64) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some(label),
        size,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub(super) struct SharpenTargets {
    pub blur_h: Texture,
    pub blur_full: Texture,
    pub sharpened_lin: Texture,
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
        let make = |label: &str, usage: TextureUsages| -> Texture {
            texture_2d(device, label, ctx.linear_format, (need_w, need_h), 1, usage)
        };
        Self {
            blur_h: make("sharpen-blur-h", STORAGE_SAMPLED),
            blur_full: make("sharpen-blur-full", STORAGE_SAMPLED),
            sharpened_lin: make("sharpened-lin", STORAGE_SAMPLED),
            post_lin: make("output-post-lin", STORAGE_SAMPLED | TextureUsages::COPY_SRC),
            alloc_w: need_w,
            alloc_h: need_h,
        }
    }
}
