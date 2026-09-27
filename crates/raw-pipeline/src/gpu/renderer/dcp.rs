use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BufferUsages, CommandEncoder, Extent3d, Texture, TextureDescriptor, TextureDimension,
    TextureUsages, TextureViewDescriptor,
};

use crate::dcp::{HsvEncoding, HueSatMap, ToneCurve};
use crate::gpu::dispatch::{bind_group, buf, copy_texture, dispatch_2d, tex};
use crate::gpu::display_depth::DisplayDepth;
use crate::gpu::passes::dcp_huesat::DcpHueSatPass;
use crate::ops::ResolvedDcp;

use super::GpuRenderer;
use super::display::DisplayTarget;
use crate::gpu::texture_pool::{PooledTexture, TextureKey};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DcpHueSatUniform {
    dims: [u32; 4],
    to_pp: [[f32; 4]; 3],
    from_pp: [[f32; 4]; 3],
    flags: [u32; 4],
    tone_lut: [[f32; 4]; 64],
}

impl DcpHueSatUniform {
    fn new(
        map: &HueSatMap,
        resolved: &ResolvedDcp,
        output: bool,
        apply_table: bool,
        tone_curve: Option<&ToneCurve>,
        warn_flags: u32,
    ) -> Self {
        let mat = |m: &[[f32; 3]; 3]| {
            [
                [m[0][0], m[0][1], m[0][2], 0.0],
                [m[1][0], m[1][1], m[1][2], 0.0],
                [m[2][0], m[2][1], m[2][2], 0.0],
            ]
        };
        let mut tone_lut = [[0.0f32; 4]; 64];
        if let Some(curve) = tone_curve {
            for i in 0..256 {
                let x = i as f32 / 255.0;
                tone_lut[i / 4][i % 4] = curve.eval(x);
            }
        }
        Self {
            dims: [
                map.hue_div,
                map.sat_div,
                map.val_div.max(1),
                matches!(map.encoding, HsvEncoding::Srgb) as u32,
            ],
            to_pp: mat(&resolved.to_pp),
            from_pp: mat(&resolved.from_pp),
            flags: [
                output as u32,
                apply_table as u32,
                tone_curve.is_some() as u32,
                warn_flags,
            ],
            tone_lut,
        }
    }
}

struct HueSatJob<'a> {
    label: &'static str,
    pass: &'a DcpHueSatPass,
    src: &'a Texture,
    map: &'a HueSatMap,
    dst: &'a Texture,
    dims: (u32, u32),
    uniform: DcpHueSatUniform,
}

pub(super) fn identity_huesat_map() -> &'static HueSatMap {
    static MAP: std::sync::OnceLock<HueSatMap> = std::sync::OnceLock::new();
    MAP.get_or_init(|| HueSatMap {
        hue_div: 1,
        sat_div: 1,
        val_div: 1,
        encoding: HsvEncoding::Linear,
        data: vec![[0.0, 1.0, 1.0]],
    })
}

impl GpuRenderer {
    pub(super) fn encode_dcp_base_table(
        &self,
        encoder: &mut CommandEncoder,
        resolved: Option<&ResolvedDcp>,
        linear_texture: &Texture,
        dims: (u32, u32),
    ) -> Option<PooledTexture> {
        let resolved = resolved?;
        let map = resolved.base_table.as_ref()?;
        let job = HueSatJob {
            label: "dcp-huesat-scratch",
            pass: &self.passes.dcp_huesat,
            src: linear_texture,
            map,
            dst: linear_texture,
            dims,
            uniform: DcpHueSatUniform::new(map, resolved, false, true, None, 0),
        };
        Some(self.apply_huesat(encoder, job))
    }

    pub(super) fn encode_dcp_finish(
        &self,
        encoder: &mut CommandEncoder,
        resolved: Option<&ResolvedDcp>,
        post_lin: &Texture,
        dst: DisplayTarget<'_>,
        warn_flags: u32,
    ) -> Option<PooledTexture> {
        let resolved = resolved?;
        let tone = resolved.tone_curve.as_deref();
        if resolved.look_table.is_none() && tone.is_none() {
            return None;
        }
        let map = resolved
            .look_table
            .as_deref()
            .unwrap_or_else(|| identity_huesat_map());
        let pass = match dst.depth {
            DisplayDepth::Eight => &self.passes.dcp_look,
            DisplayDepth::Sixteen => &self.passes.depth16(&self.ctx).dcp_look,
        };
        let apply_table = resolved.look_table.is_some();
        let job = HueSatJob {
            label: "dcp-finish-scratch",
            pass,
            src: post_lin,
            map,
            dst: dst.texture,
            dims: dst.dims,
            uniform: DcpHueSatUniform::new(map, resolved, true, apply_table, tone, warn_flags),
        };
        Some(self.apply_huesat(encoder, job))
    }

    fn apply_huesat(&self, encoder: &mut CommandEncoder, job: HueSatJob<'_>) -> PooledTexture {
        let device = &self.ctx.device;
        let (w, h) = job.dims;
        let table_tex = self.get_or_upload_huesat_texture(job.map);
        let scratch = self.texture_pool.acquire(
            device,
            TextureKey::new(
                job.dst.format(),
                w,
                h,
                1,
                TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            ),
            job.label,
        );
        let src_view = job.src.create_view(&TextureViewDescriptor::default());
        let table_view = table_tex.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D3),
            ..Default::default()
        });
        let dst_view = scratch.create_view(&TextureViewDescriptor::default());
        let ub = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("dcp-huesat-uniform"),
            contents: bytemuck::bytes_of(&job.uniform),
            usage: BufferUsages::UNIFORM,
        });
        let bg = bind_group(
            device,
            "dcp-huesat-bg",
            &job.pass.layout,
            &[buf(&ub), tex(&src_view), tex(&table_view), tex(&dst_view)],
        );
        dispatch_2d(
            encoder,
            "dcp-huesat",
            &job.pass.pipeline,
            &bg,
            w.div_ceil(16),
            h.div_ceil(16),
        );
        copy_texture(encoder, scratch.texture(), job.dst, job.dims);
        scratch
    }

    fn get_or_upload_huesat_texture(&self, map: &HueSatMap) -> Arc<Texture> {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        map.hue_div.hash(&mut hasher);
        map.sat_div.hash(&mut hasher);
        map.val_div.hash(&mut hasher);
        for px in &map.data {
            px[0].to_bits().hash(&mut hasher);
            px[1].to_bits().hash(&mut hasher);
            px[2].to_bits().hash(&mut hasher);
        }
        let key = hasher.finish();
        if let Some(t) = self.huesat_tex_cache.lock().get(&key).cloned() {
            return t;
        }
        let hue = map.hue_div;
        let sat = map.sat_div;
        let val = map.val_div.max(1);
        let rgba: Vec<f32> = map
            .data
            .iter()
            .flat_map(|px| [px[0], px[1], px[2], 0.0])
            .collect();
        let tex = self.ctx.device.create_texture(&TextureDescriptor {
            label: Some("dcp-huesat-3d"),
            size: Extent3d {
                width: hue,
                height: sat,
                depth_or_array_layers: val,
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
                bytes_per_row: Some(hue * 16),
                rows_per_image: Some(sat),
            },
            Extent3d {
                width: hue,
                height: sat,
                depth_or_array_layers: val,
            },
        );
        let tex = Arc::new(tex);
        self.huesat_tex_cache.lock().put(key, tex.clone());
        tex
    }
}
