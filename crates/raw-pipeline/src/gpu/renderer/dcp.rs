use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, BufferUsages, CommandEncoder,
    ComputePassDescriptor, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureUsages,
    TextureViewDescriptor,
};

use crate::dcp::{HsvEncoding, HueSatMap};
use crate::ops::ResolvedDcp;

use super::GpuRenderer;
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
        tone_curve: Option<&[[f32; 2]]>,
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
                tone_lut[i / 4][i % 4] = crate::color::eval_tone_curve(curve, x);
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
                0,
            ],
            tone_lut,
        }
    }
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
    pub(super) fn run_dcp_base_table(
        &self,
        encoder: &mut CommandEncoder,
        resolved: Option<&ResolvedDcp>,
        linear_texture: &Texture,
        out_w: u32,
        out_h: u32,
        sharpen_preview: bool,
    ) -> Option<PooledTexture> {
        if sharpen_preview {
            return None;
        }
        let resolved = resolved?;
        let map = resolved.base_table.as_ref()?;
        let table_tex = self.get_or_upload_huesat_texture(map);
        let scratch = self.texture_pool.acquire(
            &self.ctx.device,
            TextureKey::new(
                wgpu::TextureFormat::Rgba16Float,
                out_w,
                out_h,
                1,
                TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            ),
            "dcp-huesat-scratch",
        );
        self.encode_dcp_huesat(
            encoder,
            linear_texture,
            &table_tex,
            &scratch,
            resolved,
            map,
            out_w,
            out_h,
            false,
            true,
            None,
        );
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: scratch.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: linear_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            Extent3d {
                width: out_w,
                height: out_h,
                depth_or_array_layers: 1,
            },
        );
        Some(scratch)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn run_dcp_finish(
        &self,
        encoder: &mut CommandEncoder,
        resolved: Option<&ResolvedDcp>,
        post_lin: &Texture,
        dst: &Texture,
        out_w: u32,
        out_h: u32,
        sharpen_preview: bool,
    ) -> Option<PooledTexture> {
        if sharpen_preview {
            return None;
        }
        let resolved = resolved?;
        let tone = resolved.tone_curve.as_deref().map(Vec::as_slice);
        if resolved.look_table.is_none() && tone.is_none() {
            return None;
        }
        let map = resolved
            .look_table
            .as_deref()
            .unwrap_or_else(|| identity_huesat_map());
        let table_tex = self.get_or_upload_huesat_texture(map);
        let scratch = self.texture_pool.acquire(
            &self.ctx.device,
            TextureKey::new(
                wgpu::TextureFormat::Rgba8Unorm,
                out_w,
                out_h,
                1,
                TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            ),
            "dcp-finish-scratch",
        );
        self.encode_dcp_huesat(
            encoder,
            post_lin,
            &table_tex,
            &scratch,
            resolved,
            map,
            out_w,
            out_h,
            true,
            resolved.look_table.is_some(),
            tone,
        );
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: scratch.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: dst,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            Extent3d {
                width: out_w,
                height: out_h,
                depth_or_array_layers: 1,
            },
        );
        Some(scratch)
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

    #[allow(clippy::too_many_arguments)]
    fn encode_dcp_huesat(
        &self,
        encoder: &mut CommandEncoder,
        src: &Texture,
        table_tex: &Texture,
        dst: &Texture,
        resolved: &ResolvedDcp,
        map: &HueSatMap,
        w: u32,
        h: u32,
        output: bool,
        apply_table: bool,
        tone_curve: Option<&[[f32; 2]]>,
    ) {
        let device = &self.ctx.device;
        let pass = if output {
            &self.passes.dcp_look
        } else {
            &self.passes.dcp_huesat
        };
        let src_view = src.create_view(&TextureViewDescriptor::default());
        let table_view = table_tex.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D3),
            ..Default::default()
        });
        let dst_view = dst.create_view(&TextureViewDescriptor::default());

        let uniform = DcpHueSatUniform::new(map, resolved, output, apply_table, tone_curve);
        let ub = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("dcp-huesat-uniform"),
            contents: bytemuck::bytes_of(&uniform),
            usage: BufferUsages::UNIFORM,
        });
        let bg = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dcp-huesat-bg"),
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
                    resource: BindingResource::TextureView(&table_view),
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
            label: Some("dcp-huesat"),
            timestamp_writes: None,
        });
        cp.set_pipeline(&pass.pipeline);
        cp.set_bind_group(0, &bg, &[]);
        cp.dispatch_workgroups(gx, gy, 1);
    }
}
