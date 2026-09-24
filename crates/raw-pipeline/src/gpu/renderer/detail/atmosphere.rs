use wgpu::{BufferUsages, CommandEncoderDescriptor, Extent3d, Texture};

use crate::PipelineResult;
use crate::gpu::renderer::GpuRenderer;

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn atmosphere_for(
        &self,
        key: u64,
        src: &Texture,
        dims: (u32, u32),
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<[f32; 3]> {
        if let Some(a) = self.sensor.atmospheres.lock().get(&key).copied() {
            tracing::debug!(target: "dehaze", "atm cache hit");
            return Ok(a);
        }
        let _span = tracing::debug_span!("gpu_dehaze_atm", w = dims.0, h = dims.1).entered();
        let atm = self.estimate_atmosphere(src, dims, cancel)?;
        self.sensor
            .atmosphere_estimates
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.sensor.atmospheres.lock().put(key, atm);
        Ok(atm)
    }

    fn estimate_atmosphere(
        &self,
        src: &Texture,
        dims: (u32, u32),
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<[f32; 3]> {
        let (w, h) = dims;
        let max_dim = w.max(h);
        let level: u32 = if max_dim <= 256 {
            0
        } else {
            (max_dim as f32 / 256.0).log2().ceil() as u32
        };
        let level = level.min(src.mip_level_count().saturating_sub(1));
        let wl = (w >> level).max(1);
        let hl = (h >> level).max(1);
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
            Extent3d {
                width: wl,
                height: hl,
                depth_or_array_layers: 1,
            },
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
