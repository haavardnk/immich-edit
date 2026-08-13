use parking_lot::{Mutex, MutexGuard};
use wgpu::Texture;

use super::GpuRenderer;
use crate::gpu::context::GpuContext;
use crate::gpu::resources::{OutputTargets, SharpenTargets};

const TARGET_POOL_CAP: usize = 2;

pub(super) trait PoolTarget: Sized {
    fn fits(&self, w: u32, h: u32) -> bool;
    fn allocate(ctx: &GpuContext, w: u32, h: u32) -> Self;
}

impl PoolTarget for OutputTargets {
    fn fits(&self, w: u32, h: u32) -> bool {
        OutputTargets::fits(self, w, h)
    }

    fn allocate(ctx: &GpuContext, w: u32, h: u32) -> Self {
        OutputTargets::allocate(ctx, w, h)
    }
}

impl PoolTarget for SharpenTargets {
    fn fits(&self, w: u32, h: u32) -> bool {
        SharpenTargets::fits(self, w, h)
    }

    fn allocate(ctx: &GpuContext, w: u32, h: u32) -> Self {
        SharpenTargets::allocate(ctx, w, h)
    }
}

pub(super) fn acquire_target<'a, T: PoolTarget>(
    pool: &'a Mutex<Vec<T>>,
    ctx: &GpuContext,
    w: u32,
    h: u32,
) -> MutexGuard<'a, Vec<T>> {
    let mut guard = pool.lock();
    match guard.iter().position(|t| t.fits(w, h)) {
        Some(0) => {}
        Some(i) => {
            let t = guard.remove(i);
            guard.insert(0, t);
        }
        None => {
            if guard.len() >= TARGET_POOL_CAP {
                guard.pop();
            }
            guard.insert(0, T::allocate(ctx, w, h));
        }
    }
    guard
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GpuPoolStats {
    pub texture_pool: u64,
    pub uniform_pool: u64,
    pub output_targets: u64,
    pub sharpen_targets: u64,
    pub wb_cache: u64,
    pub nr_cache: u64,
    pub capture_cache: u64,
    pub atlas_cache: u64,
}

fn texture_bytes(tex: &Texture) -> u64 {
    let bpp = tex.format().block_copy_size(None).unwrap_or(0) as u64;
    let w = tex.width() as u64;
    let h = tex.height() as u64;
    (0..tex.mip_level_count())
        .map(|level| (w >> level).max(1) * (h >> level).max(1) * bpp)
        .sum()
}

fn output_targets_bytes(o: &OutputTargets) -> u64 {
    texture_bytes(&o.texture)
        + o.readback.size()
        + texture_bytes(&o.linear_texture)
        + o.linear_readback.size()
        + texture_bytes(&o.mask_accum_alt)
        + texture_bytes(&o.mask_base_linear)
        + texture_bytes(&o.mask_scratch_linear)
        + texture_bytes(&o.mask_scratch_tone)
        + texture_bytes(&o.mask_weight)
}

fn sharpen_targets_bytes(s: &SharpenTargets) -> u64 {
    texture_bytes(&s.blur_h)
        + texture_bytes(&s.blur_full)
        + texture_bytes(&s.sharpened_lin)
        + texture_bytes(&s.post_lin)
}

impl GpuRenderer {
    pub fn pool_stats(&self) -> GpuPoolStats {
        GpuPoolStats {
            texture_pool: self.texture_pool.bytes(),
            uniform_pool: self.uniform_pool.bytes(),
            output_targets: self
                .output_pool
                .lock()
                .iter()
                .map(output_targets_bytes)
                .sum(),
            sharpen_targets: self
                .sharpen_pool
                .lock()
                .iter()
                .map(sharpen_targets_bytes)
                .sum(),
            wb_cache: self
                .wb_cache
                .lock()
                .iter()
                .map(|(_, t)| texture_bytes(t))
                .sum(),
            nr_cache: self
                .nr_cache
                .lock()
                .iter()
                .map(|(_, t)| texture_bytes(t))
                .sum(),
            capture_cache: self
                .capture_cache
                .lock()
                .iter()
                .map(|(_, t)| texture_bytes(t))
                .sum(),
            atlas_cache: self
                .atlas_cache
                .lock()
                .iter()
                .map(|(_, v)| v.len() as u64)
                .sum(),
        }
    }
}
