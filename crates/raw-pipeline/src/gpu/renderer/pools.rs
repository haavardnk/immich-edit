use parking_lot::{Mutex, MutexGuard};
use wgpu::Texture;

use super::GpuRenderer;
#[cfg(not(feature = "native"))]
use crate::PipelineError;
use crate::PipelineResult;
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
) -> PipelineResult<MutexGuard<'a, Vec<T>>> {
    let mut guard = lock_pool(pool)?;
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
    Ok(guard)
}

#[cfg(feature = "native")]
fn lock_pool<T>(pool: &Mutex<Vec<T>>) -> PipelineResult<MutexGuard<'_, Vec<T>>> {
    Ok(pool.lock())
}

#[cfg(not(feature = "native"))]
fn lock_pool<T>(pool: &Mutex<Vec<T>>) -> PipelineResult<MutexGuard<'_, Vec<T>>> {
    pool.try_lock()
        .ok_or_else(|| PipelineError::Render("a display frame is still in flight".into()))
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
    pub atlas_pool: u64,
}

pub(super) fn texture_bytes(tex: &Texture) -> u64 {
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
        + o.histogram_counts.size()
        + o.scope_counts.size()
        + o.meta_readback.size()
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
        let [wb_cache, nr_cache, capture_cache] = self.stage_cache_bytes();
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
            wb_cache,
            nr_cache,
            capture_cache,
            atlas_cache: self
                .atlas_cache
                .lock()
                .iter()
                .map(|(_, v)| v.len() as u64)
                .sum(),
            atlas_pool: self
                .atlas_pool
                .lock()
                .iter()
                .map(|a| texture_bytes(&a.texture))
                .sum(),
        }
    }

    #[cfg(feature = "native")]
    fn stage_cache_bytes(&self) -> [u64; 3] {
        [super::Stage::Wb, super::Stage::Nr, super::Stage::Capture]
            .map(|stage| self.sensor.stages.bytes(stage))
    }

    #[cfg(not(feature = "native"))]
    fn stage_cache_bytes(&self) -> [u64; 3] {
        [0; 3]
    }
}
