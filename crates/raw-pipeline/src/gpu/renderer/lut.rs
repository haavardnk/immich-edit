use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BufferUsages, CommandEncoder, Extent3d, Texture, TextureDescriptor, TextureDimension,
    TextureUsages, TextureViewDescriptor,
};

use crate::edits::Edits;
use crate::frame::{OutputColorSpace, RenderOptions};
use crate::gpu::dispatch::{bind_group, buf, dispatch_2d, tex};
use crate::gpu::display_depth::DisplayDepth;
use crate::gpu::passes::lut::{LutParams, SHAPER_ROW};
use crate::gpu::texture::{STORAGE_SAMPLED, full_view};
use crate::gpu::texture_pool::{PooledTexture, TextureKey};
use crate::lut::{CubeLut, Domain, Lut1d, Lut3d};

use super::GpuRenderer;
use super::display::DisplayTarget;

pub(super) struct LutTextures {
    cube: Texture,
    shaper: Texture,
}

impl GpuRenderer {
    pub(super) fn maybe_encode_lut(
        &self,
        encoder: &mut CommandEncoder,
        edits: &Edits,
        opts: &RenderOptions,
        src: DisplayTarget<'_>,
    ) -> Option<PooledTexture> {
        let l = &edits.color.lut_3d;
        if !l.is_active() {
            return None;
        }
        let id = l.lut_id.as_ref()?;
        let lut = opts.luts.get(id)?;
        let textures = self.get_or_upload_lut_textures(id, lut);
        let (w, h) = src.dims;
        let target = self.texture_pool.acquire(
            &self.ctx.device,
            TextureKey::new(
                src.depth.format(),
                w,
                h,
                1,
                STORAGE_SAMPLED | TextureUsages::COPY_SRC,
            ),
            "lut-target",
        );
        let cube_domain = lut.cube().map_or(Domain::UNIT, Lut3d::domain);
        let shaper_domain = lut.shaper().map_or(Domain::UNIT, Lut1d::domain);
        let params = LutParams {
            size: [w, h],
            cube_size: lut.cube().map_or(0, |c| c.size() as u32),
            shaper_size: lut.shaper().map_or(0, |s| s.size() as u32),
            cube_min: cube_domain.min,
            shaper_width: textures.shaper.width(),
            cube_max: cube_domain.max,
            amount: (l.amount / 100.0) as f32,
            shaper_min: shaper_domain.min,
            display_p3: (opts.output_color_space == OutputColorSpace::DisplayP3) as u32,
            shaper_max: shaper_domain.max,
            ..bytemuck::Zeroable::zeroed()
        };
        self.encode_lut(encoder, src, &textures, &target, &params);
        Some(target)
    }

    fn get_or_upload_lut_textures(&self, lut_id: &str, lut: &CubeLut) -> Arc<LutTextures> {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        lut_id.hash(&mut hasher);
        lut.cube().map(Lut3d::size).hash(&mut hasher);
        lut.shaper().map(Lut1d::size).hash(&mut hasher);
        let key = hasher.finish();
        if let Some(t) = self.lut_tex_cache.lock().get(&key).cloned() {
            return t;
        }
        let textures = Arc::new(LutTextures {
            cube: self.upload_cube(lut.cube()),
            shaper: self.upload_shaper(lut.shaper()),
        });
        self.lut_tex_cache.lock().put(key, textures.clone());
        textures
    }

    fn upload_cube(&self, cube: Option<&Lut3d>) -> Texture {
        let n = cube.map_or(1, |c| c.size() as u32);
        let data = cube.map_or(&[[0.0; 3]][..], Lut3d::data);
        self.upload_rgba(
            "lut-3d",
            TextureDimension::D3,
            Extent3d {
                width: n,
                height: n,
                depth_or_array_layers: n,
            },
            data,
        )
    }

    fn upload_shaper(&self, shaper: Option<&Lut1d>) -> Texture {
        let data = shaper.map_or(&[[0.0; 3]][..], Lut1d::data);
        let n = data.len() as u32;
        let width = n.min(SHAPER_ROW);
        let height = n.div_ceil(width);
        let last = data[data.len() - 1];
        let padded: Vec<[f32; 3]> = data
            .iter()
            .copied()
            .chain(std::iter::repeat_n(last, (width * height - n) as usize))
            .collect();
        self.upload_rgba(
            "lut-shaper",
            TextureDimension::D2,
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            &padded,
        )
    }

    fn upload_rgba(
        &self,
        label: &str,
        dimension: TextureDimension,
        size: Extent3d,
        data: &[[f32; 3]],
    ) -> Texture {
        let rgba: Vec<f32> = data
            .iter()
            .flat_map(|px| [px[0], px[1], px[2], 1.0])
            .collect();
        let tex = self.ctx.device.create_texture(&TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension,
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
                bytes_per_row: Some(size.width * 16),
                rows_per_image: Some(size.height),
            },
            size,
        );
        tex
    }

    fn encode_lut(
        &self,
        encoder: &mut CommandEncoder,
        src: DisplayTarget<'_>,
        textures: &LutTextures,
        dst: &Texture,
        params: &LutParams,
    ) {
        let device = &self.ctx.device;
        let (w, h) = src.dims;
        let pass = match src.depth {
            DisplayDepth::Eight => &self.passes.lut,
            DisplayDepth::Sixteen => &self.passes.depth16(&self.ctx).lut,
        };
        let src_view = full_view(src.texture);
        let lut_view = textures.cube.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D3),
            ..Default::default()
        });
        let shaper_view = full_view(&textures.shaper);
        let dst_view = full_view(dst);
        let ub = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("lut-uniform"),
            contents: bytemuck::bytes_of(params),
            usage: BufferUsages::UNIFORM,
        });
        let bg = bind_group(
            device,
            "lut-bg",
            &pass.layout,
            &[
                buf(&ub),
                tex(&src_view),
                tex(&lut_view),
                tex(&dst_view),
                tex(&shaper_view),
            ],
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
