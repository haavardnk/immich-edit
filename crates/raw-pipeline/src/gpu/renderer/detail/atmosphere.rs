use wgpu::{BufferUsages, CommandEncoderDescriptor, Texture};

use super::pyramid::level_dims;
use crate::PipelineResult;
use crate::cancel::CancelToken;
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::texture::extent_2d;

pub(in crate::gpu::renderer) fn atmosphere_level((w, h): (u32, u32)) -> u32 {
    let max_dim = w.max(h);
    if max_dim <= 256 {
        return 0;
    }
    (max_dim as f32 / 256.0).log2().ceil() as u32
}

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn cached_atmosphere(&self, key: u64) -> Option<[f32; 3]> {
        self.sensor.atmospheres.lock().get(&key).copied()
    }

    pub(in crate::gpu::renderer) fn atmosphere_for(
        &self,
        key: u64,
        estimate: impl FnOnce() -> PipelineResult<[f32; 3]>,
    ) -> PipelineResult<[f32; 3]> {
        if let Some(a) = self.cached_atmosphere(key) {
            tracing::debug!(target: "dehaze", "atm cache hit");
            return Ok(a);
        }
        let atm = estimate()?;
        self.sensor
            .atmosphere_estimates
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.sensor.atmospheres.lock().put(key, atm);
        Ok(atm)
    }

    pub(in crate::gpu::renderer) fn estimate_atmosphere(
        &self,
        src: &Texture,
        (w, h): (u32, u32),
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<[f32; 3]> {
        let _span = tracing::debug_span!("gpu_dehaze_atm", w, h).entered();
        let level = atmosphere_level((w, h)).min(src.mip_level_count().saturating_sub(1));
        self.read_atmosphere(src, level, level_dims((w, h), level), cancel)
    }

    pub(in crate::gpu::renderer) fn read_atmosphere(
        &self,
        src: &Texture,
        level: u32,
        (wl, hl): (u32, u32),
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<[f32; 3]> {
        let bpp: u32 = 8;
        let row_align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let unpadded = wl * bpp;
        let rem = unpadded % row_align;
        let padded = if rem == 0 {
            unpadded
        } else {
            unpadded + (row_align - rem)
        };
        let buffer_size = (padded as u64) * (hl as u64);
        let buf = self.ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dehaze-atm-readback"),
            size: buffer_size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("dehaze-atm-enc"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: src,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(hl),
                },
            },
            extent_2d((wl, hl)),
        );
        self.ctx.queue.submit(Some(encoder.finish()));

        crate::gpu::readback::map_buffer_cancellable(&self.ctx, &buf, cancel)?;
        let slice = buf.slice(..);
        let data = crate::gpu::readback::mapped_range(&slice)?;
        let px_count = (wl * hl) as usize;
        let mut rgb = Vec::with_capacity(px_count * 3);
        let unpadded_bytes = (wl * 8) as usize;
        let padded_bytes = padded as usize;
        for row in 0..hl as usize {
            let start = row * padded_bytes;
            let row_u16: &[u16] = bytemuck::cast_slice(&data[start..start + unpadded_bytes]);
            for px in row_u16.chunks_exact(4) {
                rgb.push(half::f16::from_bits(px[0]).to_f32());
                rgb.push(half::f16::from_bits(px[1]).to_f32());
                rgb.push(half::f16::from_bits(px[2]).to_f32());
            }
        }
        drop(data);
        buf.unmap();

        Ok(crate::cpu::dehaze::atmosphere_from_rgb(
            &rgb,
            wl as usize,
            hl as usize,
        ))
    }
}
