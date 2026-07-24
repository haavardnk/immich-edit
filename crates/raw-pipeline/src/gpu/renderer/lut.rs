use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, BufferUsages, CommandEncoder,
    ComputePassDescriptor, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureUsages,
    TextureViewDescriptor,
};

use crate::edits::Edits;
use crate::frame::RenderOptions;
use crate::gpu::passes::lut::LUT_UNIFORM_SIZE;
use crate::gpu::texture_pool::{PooledTexture, TextureKey};

use super::GpuRenderer;

impl GpuRenderer {
    pub(super) fn maybe_encode_lut(
        &self,
        encoder: &mut CommandEncoder,
        edits: &Edits,
        opts: &RenderOptions,
        src: &Texture,
        w: u32,
        h: u32,
    ) -> Option<PooledTexture> {
        let l = &edits.color.lut_3d;
        if !l.is_active() {
            return None;
        }
        let id = l.lut_id.as_ref()?;
        let lut = opts.luts.get(id)?;
        let lut_tex = self.get_or_upload_lut_texture(id, lut);
        let target = self.texture_pool.acquire(
            &self.ctx.device,
            TextureKey::new(
                wgpu::TextureFormat::Rgba8Unorm,
                w,
                h,
                1,
                TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            ),
            "lut-target",
        );
        let amount = (l.amount / 100.0) as f32;
        self.encode_lut(encoder, src, &lut_tex, &target, lut, amount, w, h);
        Some(target)
    }

    fn get_or_upload_lut_texture(&self, lut_id: &str, lut: &crate::lut::Lut3d) -> Arc<Texture> {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        lut_id.hash(&mut hasher);
        (lut.size() as u32).hash(&mut hasher);
        let key = hasher.finish();
        if let Some(t) = self.lut_tex_cache.lock().get(&key).cloned() {
            return t;
        }
        let n = lut.size() as u32;
        let rgba: Vec<f32> = lut
            .data()
            .iter()
            .flat_map(|px| [px[0], px[1], px[2], 1.0])
            .collect();
        let tex = self.ctx.device.create_texture(&TextureDescriptor {
            label: Some("lut-3d"),
            size: Extent3d {
                width: n,
                height: n,
                depth_or_array_layers: n,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.ctx.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&rgba),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(n * 16),
                rows_per_image: Some(n),
            },
            Extent3d {
                width: n,
                height: n,
                depth_or_array_layers: n,
            },
        );
        let tex = Arc::new(tex);
        self.lut_tex_cache.lock().put(key, tex.clone());
        tex
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_lut(
        &self,
        encoder: &mut CommandEncoder,
        src: &Texture,
        lut_tex: &Texture,
        dst: &Texture,
        lut: &crate::lut::Lut3d,
        amount: f32,
        w: u32,
        h: u32,
    ) {
        let device = &self.ctx.device;
        let pass = &self.passes.lut;
        let src_view = src.create_view(&TextureViewDescriptor::default());
        let lut_view = lut_tex.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D3),
            ..Default::default()
        });
        let dst_view = dst.create_view(&TextureViewDescriptor::default());

        let mut bytes = [0u8; LUT_UNIFORM_SIZE as usize];
        bytes[0..4].copy_from_slice(&w.to_ne_bytes());
        bytes[4..8].copy_from_slice(&h.to_ne_bytes());
        bytes[8..12].copy_from_slice(&(lut.size() as u32).to_ne_bytes());
        let dmin = lut.domain_min();
        let dmax = lut.domain_max();
        bytes[16..20].copy_from_slice(&dmin[0].to_ne_bytes());
        bytes[20..24].copy_from_slice(&dmin[1].to_ne_bytes());
        bytes[24..28].copy_from_slice(&dmin[2].to_ne_bytes());
        bytes[32..36].copy_from_slice(&dmax[0].to_ne_bytes());
        bytes[36..40].copy_from_slice(&dmax[1].to_ne_bytes());
        bytes[40..44].copy_from_slice(&dmax[2].to_ne_bytes());
        bytes[48..52].copy_from_slice(&amount.to_ne_bytes());
        let ub = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("lut-uniform"),
            contents: &bytes,
            usage: BufferUsages::UNIFORM,
        });
        let bg = device.create_bind_group(&BindGroupDescriptor {
            label: Some("lut-bg"),
            layout: &pass.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: ub.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&src_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&lut_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&dst_view),
                },
            ],
        });
        let gx = w.div_ceil(16);
        let gy = h.div_ceil(16);
        let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("lut"),
            timestamp_writes: None,
        });
        cp.set_pipeline(&pass.pipeline);
        cp.set_bind_group(0, &bg, &[]);
        cp.dispatch_workgroups(gx, gy, 1);
    }
}
