use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, CommandEncoder, Texture, TextureUsages, TextureView};

use crate::dcp::{HsvEncoding, HueSatMap, ToneCurve};
use crate::gpu::dispatch::{bind_group, buf, copy_texture, dispatch_2d, tex};
use crate::gpu::display_depth::DisplayDepth;
use crate::gpu::passes::dcp_huesat::{DcpHueSatParams, huesat_table_view, upload_huesat_table};
use crate::gpu::texture::full_view;
use crate::gpu::uniform_pool::PooledUniform;
use crate::ops::ResolvedDcp;

use super::GpuRenderer;
use super::display::DisplayTarget;
use crate::gpu::texture_pool::{PooledTexture, TextureKey};

impl DcpHueSatParams {
    fn new(
        map: &HueSatMap,
        resolved: &ResolvedDcp,
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
            dims: map.gpu_dims(),
            to_pp: mat(&resolved.to_pp),
            from_pp: mat(&resolved.from_pp),
            flags: [
                apply_table as u32,
                tone_curve.is_some() as u32,
                warn_flags,
                0,
            ],
            tone_lut,
        }
    }
}

fn identity_huesat_map() -> &'static HueSatMap {
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
    pub(super) fn dcp_base_table_view(&self, resolved: Option<&ResolvedDcp>) -> TextureView {
        let map = resolved
            .and_then(|r| r.base_table.as_deref())
            .unwrap_or_else(|| identity_huesat_map());
        huesat_table_view(&self.get_or_upload_huesat_texture(map))
    }

    pub(super) fn encode_dcp_finish(
        &self,
        encoder: &mut CommandEncoder,
        resolved: Option<&ResolvedDcp>,
        post_lin: &Texture,
        dst: DisplayTarget<'_>,
        warn_flags: u32,
        curves: &PooledUniform,
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
        let uniform = DcpHueSatParams::new(
            map,
            resolved,
            resolved.look_table.is_some(),
            tone,
            warn_flags,
        );
        let device = &self.ctx.device;
        let (w, h) = dst.dims;
        let scratch = self.texture_pool.acquire(
            device,
            TextureKey::new(
                dst.texture.format(),
                w,
                h,
                1,
                TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            ),
            "dcp-finish-scratch",
        );
        let src_view = full_view(post_lin);
        let table_view = huesat_table_view(&self.get_or_upload_huesat_texture(map));
        let dst_view = full_view(&scratch);
        let ub = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("dcp-huesat-uniform"),
            contents: bytemuck::bytes_of(&uniform),
            usage: BufferUsages::UNIFORM,
        });
        let bg = bind_group(
            device,
            "dcp-huesat-bg",
            &pass.layout,
            &[
                buf(&ub),
                tex(&src_view),
                tex(&table_view),
                tex(&dst_view),
                curves.as_entire_binding(),
            ],
        );
        dispatch_2d(
            encoder,
            "dcp-huesat",
            &pass.pipeline,
            &bg,
            w.div_ceil(16),
            h.div_ceil(16),
        );
        copy_texture(encoder, scratch.texture(), dst.texture, dst.dims);
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
        let tex = Arc::new(upload_huesat_table(&self.ctx, map));
        self.huesat_tex_cache.lock().put(key, tex.clone());
        tex
    }
}
