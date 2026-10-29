use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::Ordering;

use wgpu::{CommandEncoderDescriptor, Texture, TextureFormat, TextureUsages};

use crate::PipelineResult;
use crate::ai_denoise::{DenoiseMix, DenoisePlane};
use crate::frame::RawFrame;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::denoise_blend::DenoiseBlendParams;
use crate::gpu::renderer::source_io::texel_bytes;
use crate::gpu::renderer::stage_cache::Stage;
use crate::gpu::renderer::{GpuRenderer, SensorFrame};
use crate::gpu::texture::{STORAGE_SAMPLED, mip_view, texture_2d, write_texture_2d};
use crate::tile::Rect;

fn plane_key(plane: &DenoisePlane, block: usize) -> u64 {
    let mut h = DefaultHasher::new();
    plane.key.hash(&mut h);
    plane.width.hash(&mut h);
    plane.height.hash(&mut h);
    block.hash(&mut h);
    h.finish()
}

fn blend_key(frame: &RawFrame, src: &SensorFrame, plane: &DenoisePlane, mix: &DenoiseMix) -> u64 {
    let mut h = DefaultHasher::new();
    frame.id.hash(&mut h);
    src.rect.dims().hash(&mut h);
    plane_key(plane, src.block).hash(&mut h);
    mix.hash_key(&mut h);
    h.finish()
}

impl GpuRenderer {
    pub fn denoise_blends(&self) -> u64 {
        self.sensor.denoise_blends.load(Ordering::Relaxed)
    }

    fn cached_plane_texture(&self, plane: &DenoisePlane, src: &SensorFrame) -> Arc<Texture> {
        let key = plane_key(plane, src.block);
        if let Some(texture) = self.sensor.stages.get(Stage::DenoisePlane, key) {
            return texture;
        }
        let texture = self.denoise_plane_texture(plane, src.block, src.rect);
        self.sensor
            .stages
            .put(Stage::DenoisePlane, key, texture.clone());
        texture
    }

    fn denoise_plane_texture(
        &self,
        plane: &DenoisePlane,
        block: usize,
        rect: Rect,
    ) -> Arc<Texture> {
        let (w, h) = rect.dims();
        let texture = texture_2d(
            &self.ctx.device,
            "denoise-plane",
            TextureFormat::Rgba16Float,
            (w, h),
            1,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        );
        let texels = texel_bytes(
            &plane.binned_rgb_f16(block, rect),
            TextureFormat::Rgba16Float,
        );
        write_texture_2d(&self.ctx.queue, &texture, &texels, w * 8, (w, h));
        Arc::new(texture)
    }

    pub(in crate::gpu::renderer) fn submit_denoise_blend(
        &self,
        frame: &RawFrame,
        src: &Arc<SensorFrame>,
        plane: &DenoisePlane,
        mix: DenoiseMix,
        cache: bool,
    ) -> PipelineResult<Arc<SensorFrame>> {
        let (w, h) = src.rect.dims();
        let key = cache.then(|| blend_key(frame, src, plane, &mix));
        let blended = |texture| {
            Arc::new(SensorFrame {
                texture,
                rect: src.rect,
                block: src.block,
            })
        };
        if let Some(texture) = key.and_then(|key| self.sensor.stages.get(Stage::DenoiseBlend, key))
        {
            tracing::debug!(target: "gpu_cache", "denoise_blend cache hit");
            return Ok(blended(texture));
        }
        let _span = tracing::debug_span!("gpu.submit_denoise_blend", w, h).entered();
        let device = &self.ctx.device;
        let plane_texture = if cache {
            self.cached_plane_texture(plane, src)
        } else {
            self.denoise_plane_texture(plane, src.block, src.rect)
        };
        let uniform_buf = self.uniform(
            &DenoiseBlendParams {
                size: [w, h],
                strength: mix.strength,
                detail: mix.detail,
                luma: mix.axis.weights,
                _pad0: 0.0,
                neutral: mix.axis.neutral,
                _pad1: 0.0,
            },
            "denoise-blend-uniform",
        );
        let dst = texture_2d(
            device,
            "denoise-blend-out",
            self.ctx.linear_format,
            (w, h),
            mip_count(w, h),
            STORAGE_SAMPLED | TextureUsages::COPY_SRC,
        );
        let src_view = mip_view(&src.texture, 0);
        let plane_view = mip_view(&plane_texture, 0);
        let dst_view = mip_view(&dst, 0);
        let pass = &self.passes.sensor_stage.denoise_blend;
        let bind = bind_group(
            device,
            "denoise-blend-bg",
            &pass.layout,
            &[
                uniform_buf.as_entire_binding(),
                tex(&src_view),
                tex(&plane_view),
                tex(&dst_view),
            ],
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("denoise-blend-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "denoise-blend-pass",
            &pass.pipeline,
            &bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
        self.encode_mipgen(&mut encoder, &dst, w, h);
        self.ctx.queue.submit(Some(encoder.finish()));
        self.sensor.denoise_blends.fetch_add(1, Ordering::Relaxed);
        let texture = Arc::new(dst);
        if let Some(key) = key {
            self.sensor
                .stages
                .put(Stage::DenoiseBlend, key, texture.clone());
        }
        Ok(blended(texture))
    }
}
