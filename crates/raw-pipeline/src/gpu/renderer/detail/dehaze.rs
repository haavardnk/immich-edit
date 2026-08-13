use std::sync::Arc;

use wgpu::{
    BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor, Extent3d, Texture,
    TextureUsages, TextureViewDescriptor,
};

use crate::PipelineResult;
use crate::edits::Edits;
use crate::frame::RawFrame;
use crate::gpu::dispatch::{bind_group, samp, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::dehaze::{
    DehazeApplyParams, DehazeDownsampleParams, DehazeFilterParams, DehazeNormParams, MOMENT_FORMAT,
};
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::renderer::cache_keys::atmosphere_cache_key;
use crate::gpu::texture_pool::TextureKey;

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn atmosphere_for(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        src: &Texture,
        dims: (u32, u32),
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<[f32; 3]> {
        let key = atmosphere_cache_key(frame, edits, dims);
        if let Some(a) = self.atm_cache.lock().get(&key).copied() {
            tracing::debug!(target: "dehaze", "atm cache hit");
            return Ok(a);
        }
        let _span = tracing::debug_span!("gpu_dehaze_atm", w = dims.0, h = dims.1).entered();
        let atm = self.estimate_atmosphere(src, dims, cancel)?;
        self.atm_cache.lock().put(key, atm);
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

    pub(in crate::gpu::renderer) fn run_dehaze(
        &self,
        src: &Texture,
        dims: (u32, u32),
        edits: &Edits,
        atm: [f32; 3],
    ) -> PipelineResult<Arc<Texture>> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (w, h) = dims;
        let min_dim_full = w.min(h);
        let half_min_full = (min_dim_full / 2).max(1);
        let r_patch_full: u32 = (min_dim_full / 200).max(8).min(half_min_full);
        let r_gf_full: u32 = (min_dim_full / 50).max(16).min(half_min_full);
        let scale: u32 = if min_dim_full >= 512 { 4 } else { 1 };
        let lw = (w / scale).max(1);
        let lh = (h / scale).max(1);
        let r_patch: u32 = (r_patch_full / scale).max(2);
        let r_gf: u32 = (r_gf_full / scale).max(4);
        let amount = (edits.basic.dehaze as f32 / 100.0).clamp(-1.0, 1.0);

        let scratch_key = TextureKey::new(
            self.ctx.linear_format,
            lw,
            lh,
            1,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        );
        let moment_key = TextureKey::new(
            MOMENT_FORMAT,
            lw,
            lh,
            1,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        );
        let make_scratch_lo =
            |label: &'static str| self.texture_pool.acquire(device, scratch_key, label);
        let make_moment_lo =
            |label: &'static str| self.texture_pool.acquire(device, moment_key, label);
        let lo_src = make_scratch_lo("dehaze-lo-src");
        let dn = make_moment_lo("dehaze-dn");
        let dn_h = make_moment_lo("dehaze-dn-h");
        let dn_min = make_moment_lo("dehaze-dn-min");
        let packed = make_moment_lo("dehaze-pack");
        let packed_h = make_moment_lo("dehaze-pack-h");
        let packed_v = make_moment_lo("dehaze-pack-v");
        let ab = make_moment_lo("dehaze-ab");
        let ab_h = make_moment_lo("dehaze-ab-h");
        let ab_v = make_moment_lo("dehaze-ab-v");
        let out_key = TextureKey::new(
            self.ctx.linear_format,
            w,
            h,
            mip_count(w, h),
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        );
        let out = self.texture_pool.acquire(device, out_key, "dehaze-out");

        let downsample_params = DehazeDownsampleParams {
            size: [lw, lh],
            scale,
            _pad: 0,
        };
        let downsample_buf = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&downsample_params),
            "dehaze-downsample-u",
        );

        let norm_params = DehazeNormParams {
            size: [lw, lh],
            _pad: [0; 2],
            atmosphere: [atm[0], atm[1], atm[2], 1.0],
        };
        let norm_buf = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&norm_params),
            "dehaze-norm-u",
        );

        let make_filter_u = |radius: u32, axis: u32, label: &'static str| {
            let params = DehazeFilterParams {
                size: [lw, lh],
                radius,
                axis,
            };
            self.uniform_pool
                .acquire(device, queue, bytemuck::bytes_of(&params), label)
        };
        let min_h_buf = make_filter_u(r_patch, 0, "dehaze-min-h-u");
        let min_v_buf = make_filter_u(r_patch, 1, "dehaze-min-v-u");
        let box_h_buf = make_filter_u(r_gf, 0, "dehaze-box-h-u");
        let box_v_buf = make_filter_u(r_gf, 1, "dehaze-box-v-u");

        let pack_buf = make_filter_u(0, 0, "dehaze-pack-u");
        let ab_uni = make_filter_u(0, 0, "dehaze-ab-u");

        let apply_params = DehazeApplyParams {
            size: [w, h],
            lo_size: [lw, lh],
            atmosphere: [atm[0], atm[1], atm[2], 1.0],
            amount,
            _pad: [0.0; 3],
        };
        let apply_buf = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&apply_params),
            "dehaze-apply-u",
        );

        let src_view = src.create_view(&TextureViewDescriptor::default());
        let lo_src_view = lo_src.create_view(&TextureViewDescriptor::default());
        let lo_src_store_view = lo_src.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let dn_view = dn.create_view(&TextureViewDescriptor::default());
        let dn_h_view = dn_h.create_view(&TextureViewDescriptor::default());
        let dn_min_view = dn_min.create_view(&TextureViewDescriptor::default());
        let packed_view = packed.create_view(&TextureViewDescriptor::default());
        let packed_h_view = packed_h.create_view(&TextureViewDescriptor::default());
        let packed_v_view = packed_v.create_view(&TextureViewDescriptor::default());
        let ab_view = ab.create_view(&TextureViewDescriptor::default());
        let ab_h_view = ab_h.create_view(&TextureViewDescriptor::default());
        let ab_v_view = ab_v.create_view(&TextureViewDescriptor::default());
        let out_view = out.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let p = &self.passes.dehaze;
        let bg_downsample = bind_group(
            device,
            "dehaze-downsample-bg",
            &p.downsample_layout,
            &[
                downsample_buf.as_entire_binding(),
                tex(&src_view),
                samp(&p.linear_sampler),
                tex(&lo_src_store_view),
            ],
        );
        let bg_norm = bind_group(
            device,
            "dehaze-norm-bg",
            &p.norm_layout,
            &[
                norm_buf.as_entire_binding(),
                tex(&lo_src_view),
                tex(&dn_view),
            ],
        );
        let bg_min_h = bind_group(
            device,
            "dehaze-min-h-bg",
            &p.min_layout,
            &[
                min_h_buf.as_entire_binding(),
                tex(&dn_view),
                tex(&dn_h_view),
            ],
        );
        let bg_min_v = bind_group(
            device,
            "dehaze-min-v-bg",
            &p.min_layout,
            &[
                min_v_buf.as_entire_binding(),
                tex(&dn_h_view),
                tex(&dn_min_view),
            ],
        );
        let bg_pack = bind_group(
            device,
            "dehaze-pack-bg",
            &p.pack_layout,
            &[
                pack_buf.as_entire_binding(),
                tex(&lo_src_view),
                tex(&dn_min_view),
                tex(&packed_view),
            ],
        );
        let bg_box_h_pack = bind_group(
            device,
            "dehaze-box-h-pack-bg",
            &p.box_layout,
            &[
                box_h_buf.as_entire_binding(),
                tex(&packed_view),
                tex(&packed_h_view),
            ],
        );
        let bg_box_v_pack = bind_group(
            device,
            "dehaze-box-v-pack-bg",
            &p.box_layout,
            &[
                box_v_buf.as_entire_binding(),
                tex(&packed_h_view),
                tex(&packed_v_view),
            ],
        );
        let bg_ab = bind_group(
            device,
            "dehaze-ab-bg",
            &p.ab_layout,
            &[
                ab_uni.as_entire_binding(),
                tex(&packed_v_view),
                tex(&ab_view),
            ],
        );
        let bg_box_h_ab = bind_group(
            device,
            "dehaze-box-h-ab-bg",
            &p.box_layout,
            &[
                box_h_buf.as_entire_binding(),
                tex(&ab_view),
                tex(&ab_h_view),
            ],
        );
        let bg_box_v_ab = bind_group(
            device,
            "dehaze-box-v-ab-bg",
            &p.box_layout,
            &[
                box_v_buf.as_entire_binding(),
                tex(&ab_h_view),
                tex(&ab_v_view),
            ],
        );
        let bg_apply = bind_group(
            device,
            "dehaze-apply-bg",
            &p.apply_layout,
            &[
                apply_buf.as_entire_binding(),
                tex(&src_view),
                tex(&ab_v_view),
                tex(&out_view),
            ],
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("dehaze-enc"),
        });
        let gx_lo = lw.div_ceil(16);
        let gy_lo = lh.div_ceil(16);
        let gx = w.div_ceil(16);
        let gy = h.div_ceil(16);
        {
            let mut c = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("dehaze-pass"),
                timestamp_writes: None,
            });
            c.set_pipeline(&p.downsample_pipeline);
            c.set_bind_group(0, &bg_downsample, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.norm_pipeline);
            c.set_bind_group(0, &bg_norm, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.min_pipeline);
            c.set_bind_group(0, &bg_min_h, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_bind_group(0, &bg_min_v, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.pack_pipeline);
            c.set_bind_group(0, &bg_pack, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.box_pipeline);
            c.set_bind_group(0, &bg_box_h_pack, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_bind_group(0, &bg_box_v_pack, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.ab_pipeline);
            c.set_bind_group(0, &bg_ab, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.box_pipeline);
            c.set_bind_group(0, &bg_box_h_ab, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_bind_group(0, &bg_box_v_ab, &[]);
            c.dispatch_workgroups(gx_lo, gy_lo, 1);
            c.set_pipeline(&p.apply_pipeline);
            c.set_bind_group(0, &bg_apply, &[]);
            c.dispatch_workgroups(gx, gy, 1);
        }
        self.encode_mipgen(&mut encoder, &out, w, h);
        queue.submit(Some(encoder.finish()));
        Ok(out.into_arc())
    }
}
