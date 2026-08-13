use std::sync::Arc;

use wgpu::{
    CommandEncoderDescriptor, Extent3d, Texture, TextureDescriptor, TextureDimension,
    TextureUsages, TextureViewDescriptor,
};

use crate::PipelineResult;
use crate::edits::Edits;
use crate::frame::RawFrame;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::nr::NrParams;
use crate::gpu::passes::nr_smooth::NrSmoothParams;
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::renderer::cache_keys::nr_cache_key;

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn run_nr(
        &self,
        src: &Texture,
        dims: (u32, u32),
        edits: &Edits,
        frame: &RawFrame,
        cam_to_srgb: [[f32; 3]; 3],
    ) -> PipelineResult<Arc<Texture>> {
        let key = nr_cache_key(frame, edits, dims, cam_to_srgb);
        if let Some(t) = self.nr_cache.lock().get(&key).cloned() {
            tracing::debug!(target: "gpu_cache", "nr_out cache hit");
            return Ok(t);
        }
        let _span = tracing::debug_span!("gpu.run_nr", w = dims.0, h = dims.1).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (w, h) = dims;
        let d = &edits.detail;

        let luma_amount = d.luma_nr_amount as f32;
        let luma_detail = d.luma_nr_detail as f32;
        let luma_contrast = d.luma_nr_contrast as f32;
        let color_amount = d.color_nr_amount as f32;
        let color_detail = d.color_nr_detail as f32;

        let radius_for = |amount: f32| -> u32 {
            if amount >= 66.0 {
                4
            } else if amount >= 33.0 {
                3
            } else {
                2
            }
        };
        let sigma_r_luma = 0.005 + (1.0 - luma_detail / 100.0) * 0.20;
        let sigma_r_chroma = 0.005 + (1.0 - color_detail / 100.0) * 0.30;
        let inv_2sr_luma = 1.0 / (2.0 * sigma_r_luma * sigma_r_luma);
        let inv_2sr_chroma = 1.0 / (2.0 * sigma_r_chroma * sigma_r_chroma);
        let alpha_luma = luma_amount / 100.0;
        let alpha_chroma = color_amount / 100.0;
        let contrast = luma_contrast / 100.0;

        let nr_uniform = |stage: u32, radius: u32| {
            let sigma_s = radius as f32;
            let params = NrParams {
                size: [w, h],
                radius,
                stage,
                inv_2ss: 1.0 / (2.0 * sigma_s * sigma_s),
                inv_2sr_luma,
                inv_2sr_chroma,
                alpha_luma,
                alpha_chroma,
                contrast,
                _pad: [0.0; 2],
            };
            self.uniform_pool
                .acquire(device, queue, bytemuck::bytes_of(&params), "nr-uniform")
        };

        let make_tex = |label: &'static str, mips: bool| -> Texture {
            device.create_texture(&TextureDescriptor {
                label: Some(label),
                size: Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: if mips { mip_count(w, h) } else { 1 },
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: self.ctx.linear_format,
                usage: TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };

        let chroma_active = color_amount > 0.0;
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("nr-enc"),
        });
        let dispatch = |encoder: &mut wgpu::CommandEncoder,
                        label: &'static str,
                        uniform: &crate::gpu::uniform_pool::PooledUniform,
                        src: &wgpu::TextureView,
                        dst: &wgpu::TextureView| {
            let bind = bind_group(
                device,
                label,
                &self.passes.nr.layout,
                &[uniform.as_entire_binding(), tex(src), tex(dst)],
            );
            dispatch_2d(
                encoder,
                label,
                &self.passes.nr.pipeline,
                &bind,
                w.div_ceil(16),
                h.div_ceil(16),
            );
        };

        let src_view = src.create_view(&TextureViewDescriptor::default());
        let luma_tex = (luma_amount > 0.0).then(|| make_tex("nr-luma", !chroma_active));
        let luma_uniform = luma_tex
            .as_ref()
            .map(|_| nr_uniform(0, radius_for(luma_amount)));
        if let (Some(t), Some(u)) = (luma_tex.as_ref(), luma_uniform.as_ref()) {
            let view = t.create_view(&TextureViewDescriptor {
                base_mip_level: 0,
                mip_level_count: Some(1),
                ..Default::default()
            });
            dispatch(&mut encoder, "nr-luma-pass", u, &src_view, &view);
        }

        if !chroma_active {
            let Some(dst) = luma_tex else {
                queue.submit(Some(encoder.finish()));
                let out = Arc::new(make_tex("nr-out", true));
                return Ok(out);
            };
            self.encode_mipgen(&mut encoder, &dst, w, h);
            queue.submit(Some(encoder.finish()));
            let out = Arc::new(dst);
            self.nr_cache.lock().put(key, out.clone());
            return Ok(out);
        }

        let base_view = match luma_tex.as_ref() {
            Some(t) => t.create_view(&TextureViewDescriptor::default()),
            None => src.create_view(&TextureViewDescriptor::default()),
        };
        let chroma_tex = make_tex("nr-chroma", false);
        let chroma_view = chroma_tex.create_view(&TextureViewDescriptor::default());
        let u = nr_uniform(1, radius_for(color_amount));
        dispatch(&mut encoder, "nr-chroma-pass", &u, &base_view, &chroma_view);

        let dst = make_tex("nr-out", true);
        let dst_mip0 = dst.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let smoothness = (d.color_nr_smoothness as f32) / 100.0;
        let smooth_params = NrSmoothParams {
            size: [w, h],
            _pad0: [0; 2],
            smoothness,
            alpha_chroma,
            _pad1: [0.0; 6],
        };
        let sbuf = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&smooth_params),
            "nr-smooth-uniform",
        );
        let sbind = bind_group(
            device,
            "nr-smooth-bg",
            &self.passes.nr_smooth.layout,
            &[
                sbuf.as_entire_binding(),
                tex(&base_view),
                tex(&chroma_view),
                tex(&dst_mip0),
            ],
        );
        dispatch_2d(
            &mut encoder,
            "nr-chroma-finish",
            &self.passes.nr_smooth.pipeline,
            &sbind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
        self.encode_mipgen(&mut encoder, &dst, w, h);
        queue.submit(Some(encoder.finish()));
        let out = Arc::new(dst);
        self.nr_cache.lock().put(key, out.clone());
        Ok(out)
    }
}
