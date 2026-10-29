use wgpu::{Buffer, BufferDescriptor, BufferUsages, Limits, Texture, TextureFormat, TextureUsages};

use super::context::GpuContext;
use super::display_depth::DisplayDepth;
use super::helpers::round_up_256;
use super::limits::{check_buffer, check_texture};
use super::passes::meta_bins::HISTOGRAM_COUNTS;
use super::readback::{make_readback_buffer, readback_size};
use super::texture::{STORAGE_SAMPLED, texture_2d};
use crate::PipelineResult;
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
    pub alloc_w: u32,
    pub alloc_h: u32,
}

impl OutputTargets {
    pub fn check_limits(
        limits: &Limits,
        out_dims: (u32, u32),
        depth: DisplayDepth,
    ) -> PipelineResult<()> {
        let (alloc_w, alloc_h) = target_dims(out_dims);
        check_texture(limits, "output", (alloc_w, alloc_h))?;
        check_buffer(
            limits,
            "output readback",
            readback_size(alloc_w, alloc_h, 4),
        )?;
        if depth == DisplayDepth::Sixteen {
            let (out_w, out_h) = out_dims;
            check_buffer(limits, "16-bit readback", readback_size(out_w, out_h, 8))?;
        }
        Ok(())
    }

    pub fn fits(&self, w: u32, h: u32) -> bool {
        self.alloc_w >= w && self.alloc_h >= h
    }

    pub fn allocate(ctx: &GpuContext, out_w: u32, out_h: u32) -> Self {
        let device = &ctx.device;
        let (need_w, need_h) = target_dims((out_w, out_h));
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
            alloc_w: need_w,
            alloc_h: need_h,
        }
    }
}

fn target_dims(out_dims: (u32, u32)) -> (u32, u32) {
    (round_up_256(out_dims.0), round_up_256(out_dims.1))
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
        let (need_w, need_h) = target_dims((out_w, out_h));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::limits::unsupported;

    #[test]
    fn output_targets_are_checked_at_their_rounded_allocation() {
        let limits = Limits {
            max_texture_dimension_2d: 1000,
            ..Limits::default()
        };
        if !unsupported(OutputTargets::check_limits(
            &limits,
            (1000, 10),
            DisplayDepth::Eight,
        )) {
            panic!("a 1000 px output allocates 1024 px and must not fit a 1000 px limit");
        }
    }

    #[test]
    fn a_sixteen_bit_output_also_checks_its_wide_readback() {
        let limits = Limits {
            max_texture_dimension_2d: 1024,
            max_buffer_size: 4 << 20,
            ..Limits::default()
        };
        if let Err(e) = OutputTargets::check_limits(&limits, (1000, 1000), DisplayDepth::Eight) {
            panic!("an 8-bit 1000x1000 output with a 4 MiB readback was refused: {e}");
        }
        if !unsupported(OutputTargets::check_limits(
            &limits,
            (1000, 1000),
            DisplayDepth::Sixteen,
        )) {
            panic!("a 16-bit 1000x1000 readback needs 8 MB and must not fit 4 MiB");
        }
    }
}
