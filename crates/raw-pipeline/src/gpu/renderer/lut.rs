use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BufferUsages, CommandEncoder, Extent3d, Texture, TextureDescriptor, TextureDimension,
    TextureUsages, TextureViewDescriptor,
};

use crate::edits::Edits;
use crate::frame::RenderOptions;
use crate::gpu::dispatch::{bind_group, buf, dispatch_2d, tex};
use crate::gpu::passes::lut::LutParams;
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

        let dmin = lut.domain_min();
        let dmax = lut.domain_max();
        let params = LutParams {
            size: [w, h, lut.size() as u32],
            domain_min: dmin,
            domain_max: dmax,
            amount,
            ..bytemuck::Zeroable::zeroed()
        };
        let ub = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("lut-uniform"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM,
        });
        let bg = bind_group(
            device,
            "lut-bg",
            &pass.layout,
            &[buf(&ub), tex(&src_view), tex(&lut_view), tex(&dst_view)],
        );
        dispatch_2d(
            encoder,
            "lut",
            &pass.pipeline,
            &bg,
            w.div_ceil(16),
            h.div_ceil(16),
        );
    }
}
