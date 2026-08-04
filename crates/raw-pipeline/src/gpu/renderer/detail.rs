use std::sync::Arc;

use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, BufferUsages, CommandEncoderDescriptor,
    ComputePassDescriptor, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureUsages,
    TextureViewDescriptor,
};

use crate::PipelineResult;
use crate::edits::Edits;
use crate::frame::RawFrame;
use crate::gpu::passes::luma_pyramid::LumaPyramidPass;
use crate::gpu::passes::presence::PRESENCE_UNIFORM_SIZE;
use crate::gpu::texture_pool::TextureKey;
use crate::presence::{presence_amounts, presence_mips, presence_pyramid_levels, presence_radii};

use crate::gpu::helpers::mip_count;
use crate::gpu::uniforms::{write_active_mask, write_header};
use crate::ops::{OpContext, OpScratch, RenderContext};

use super::{CachedFrame, GpuRenderer, atmosphere_cache_key, nr_cache_key, wb_cache_key};

impl GpuRenderer {
    pub(super) fn run_nr(
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

        let max_amount = luma_amount.max(color_amount);
        let radius: u32 = if max_amount >= 66.0 {
            4
        } else if max_amount >= 33.0 {
            3
        } else {
            2
        };
        let sigma_s = radius as f32;
        let sigma_r_luma = 0.005 + (1.0 - luma_detail / 100.0) * 0.20;
        let sigma_r_chroma = 0.005 + (1.0 - color_detail / 100.0) * 0.30;
        let inv_2ss = 1.0 / (2.0 * sigma_s * sigma_s);
        let inv_2sr_luma = 1.0 / (2.0 * sigma_r_luma * sigma_r_luma);
        let inv_2sr_chroma = 1.0 / (2.0 * sigma_r_chroma * sigma_r_chroma);
        let alpha_luma = luma_amount / 100.0;
        let alpha_chroma = color_amount / 100.0;
        let contrast = luma_contrast / 100.0;

        let mut bytes = vec![0u8; crate::gpu::passes::nr::NR_UNIFORM_SIZE as usize];
        bytes[0..4].copy_from_slice(&w.to_le_bytes());
        bytes[4..8].copy_from_slice(&h.to_le_bytes());
        bytes[8..12].copy_from_slice(&radius.to_le_bytes());
        bytes[16..20].copy_from_slice(&inv_2ss.to_le_bytes());
        bytes[20..24].copy_from_slice(&inv_2sr_luma.to_le_bytes());
        bytes[24..28].copy_from_slice(&inv_2sr_chroma.to_le_bytes());
        bytes[28..32].copy_from_slice(&alpha_luma.to_le_bytes());
        bytes[32..36].copy_from_slice(&alpha_chroma.to_le_bytes());
        bytes[36..40].copy_from_slice(&contrast.to_le_bytes());
        let uniform_buf = self
            .uniform_pool
            .acquire(device, queue, &bytes, "nr-uniform");

        let dst = device.create_texture(&TextureDescriptor {
            label: Some("nr-out"),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count(w, h),
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.ctx.linear_format,
            usage: TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let src_view = src.create_view(&TextureViewDescriptor::default());
        let dst_mip0 = dst.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("nr-bg"),
            layout: &self.passes.nr.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&src_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&dst_mip0),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("nr-enc"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("nr-pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.passes.nr.pipeline);
            cpass.set_bind_group(0, &bind, &[]);
            cpass.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
        }
        self.encode_mipgen(&mut encoder, &dst, w, h);
        queue.submit(Some(encoder.finish()));

        let smoothness = (d.color_nr_smoothness as f32) / 100.0;
        if smoothness > 0.0 && color_amount > 0.0 {
            let smoothed = self.run_nr_smooth(&dst, dims, smoothness)?;
            self.nr_cache.lock().put(key, smoothed.clone());
            return Ok(smoothed);
        }
        let out = Arc::new(dst);
        self.nr_cache.lock().put(key, out.clone());
        Ok(out)
    }

    fn run_nr_smooth(
        &self,
        src: &Texture,
        dims: (u32, u32),
        smoothness: f32,
    ) -> PipelineResult<Arc<Texture>> {
        let _span = tracing::debug_span!("gpu.run_nr_smooth", w = dims.0, h = dims.1).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (w, h) = dims;

        let mut bytes = vec![0u8; crate::gpu::passes::nr_smooth::NR_SMOOTH_UNIFORM_SIZE as usize];
        bytes[0..4].copy_from_slice(&w.to_le_bytes());
        bytes[4..8].copy_from_slice(&h.to_le_bytes());
        bytes[16..20].copy_from_slice(&smoothness.to_le_bytes());
        let uniform_buf = self
            .uniform_pool
            .acquire(device, queue, &bytes, "nr-smooth-uniform");

        let dst = device.create_texture(&TextureDescriptor {
            label: Some("nr-smooth-out"),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count(w, h),
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.ctx.linear_format,
            usage: TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let src_view = src.create_view(&TextureViewDescriptor::default());
        let dst_mip0 = dst.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("nr-smooth-bg"),
            layout: &self.passes.nr_smooth.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&src_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&dst_mip0),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("nr-smooth-enc"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("nr-smooth-pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.passes.nr_smooth.pipeline);
            cpass.set_bind_group(0, &bind, &[]);
            cpass.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
        }
        self.encode_mipgen(&mut encoder, &dst, w, h);
        queue.submit(Some(encoder.finish()));
        Ok(Arc::new(dst))
    }

    pub(super) fn atmosphere_for(
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

        let mut dp = vec![0.0f32; px_count];
        for i in 0..px_count {
            let r = rgb[i * 3].clamp(0.0, 1.0);
            let g = rgb[i * 3 + 1].clamp(0.0, 1.0);
            let b = rgb[i * 3 + 2].clamp(0.0, 1.0);
            dp[i] = r.min(g).min(b);
        }
        Ok(crate::cpu::dehaze::estimate_atmosphere(
            &rgb,
            &dp,
            wl as usize,
            hl as usize,
        ))
    }

    pub(super) fn run_dehaze(
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
        let make_scratch_lo =
            |label: &'static str| self.texture_pool.acquire(device, scratch_key, label);
        let lo_src = make_scratch_lo("dehaze-lo-src");
        let dn = make_scratch_lo("dehaze-dn");
        let dn_h = make_scratch_lo("dehaze-dn-h");
        let dn_min = make_scratch_lo("dehaze-dn-min");
        let packed = make_scratch_lo("dehaze-pack");
        let packed_h = make_scratch_lo("dehaze-pack-h");
        let packed_v = make_scratch_lo("dehaze-pack-v");
        let ab = make_scratch_lo("dehaze-ab");
        let ab_h = make_scratch_lo("dehaze-ab-h");
        let ab_v = make_scratch_lo("dehaze-ab-v");
        let out_key = TextureKey::new(
            self.ctx.linear_format,
            w,
            h,
            mip_count(w, h),
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        );
        let out = self.texture_pool.acquire(device, out_key, "dehaze-out");

        let mut downsample_u =
            vec![0u8; crate::gpu::passes::dehaze::DOWNSAMPLE_UNIFORM_SIZE as usize];
        downsample_u[0..4].copy_from_slice(&lw.to_le_bytes());
        downsample_u[4..8].copy_from_slice(&lh.to_le_bytes());
        downsample_u[8..12].copy_from_slice(&scale.to_le_bytes());
        let downsample_buf =
            self.uniform_pool
                .acquire(device, queue, &downsample_u, "dehaze-downsample-u");

        let mut norm_u = vec![0u8; 32];
        norm_u[0..4].copy_from_slice(&lw.to_le_bytes());
        norm_u[4..8].copy_from_slice(&lh.to_le_bytes());
        norm_u[16..20].copy_from_slice(&atm[0].to_le_bytes());
        norm_u[20..24].copy_from_slice(&atm[1].to_le_bytes());
        norm_u[24..28].copy_from_slice(&atm[2].to_le_bytes());
        norm_u[28..32].copy_from_slice(&1.0f32.to_le_bytes());
        let norm_buf = self
            .uniform_pool
            .acquire(device, queue, &norm_u, "dehaze-norm-u");

        let make_filter_u = |radius: u32, axis: u32, label: &'static str| {
            let mut u = vec![0u8; 16];
            u[0..4].copy_from_slice(&lw.to_le_bytes());
            u[4..8].copy_from_slice(&lh.to_le_bytes());
            u[8..12].copy_from_slice(&radius.to_le_bytes());
            u[12..16].copy_from_slice(&axis.to_le_bytes());
            self.uniform_pool.acquire(device, queue, &u, label)
        };
        let min_h_buf = make_filter_u(r_patch, 0, "dehaze-min-h-u");
        let min_v_buf = make_filter_u(r_patch, 1, "dehaze-min-v-u");
        let box_h_buf = make_filter_u(r_gf, 0, "dehaze-box-h-u");
        let box_v_buf = make_filter_u(r_gf, 1, "dehaze-box-v-u");

        let mut size_u = vec![0u8; 16];
        size_u[0..4].copy_from_slice(&lw.to_le_bytes());
        size_u[4..8].copy_from_slice(&lh.to_le_bytes());
        let pack_buf = self
            .uniform_pool
            .acquire(device, queue, &size_u, "dehaze-pack-u");
        let ab_uni = self
            .uniform_pool
            .acquire(device, queue, &size_u, "dehaze-ab-u");

        let mut apply_u = vec![0u8; 48];
        apply_u[0..4].copy_from_slice(&w.to_le_bytes());
        apply_u[4..8].copy_from_slice(&h.to_le_bytes());
        apply_u[8..12].copy_from_slice(&lw.to_le_bytes());
        apply_u[12..16].copy_from_slice(&lh.to_le_bytes());
        apply_u[16..20].copy_from_slice(&atm[0].to_le_bytes());
        apply_u[20..24].copy_from_slice(&atm[1].to_le_bytes());
        apply_u[24..28].copy_from_slice(&atm[2].to_le_bytes());
        apply_u[28..32].copy_from_slice(&1.0f32.to_le_bytes());
        apply_u[32..36].copy_from_slice(&amount.to_le_bytes());
        let apply_buf = self
            .uniform_pool
            .acquire(device, queue, &apply_u, "dehaze-apply-u");

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
        let bg_downsample = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dehaze-downsample-bg"),
            layout: &p.downsample_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: downsample_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&src_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&p.linear_sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&lo_src_store_view),
                },
            ],
        });
        let bg_norm = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dehaze-norm-bg"),
            layout: &p.norm_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: norm_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&lo_src_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&dn_view),
                },
            ],
        });
        let bg_min_h = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dehaze-min-h-bg"),
            layout: &p.min_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: min_h_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&dn_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&dn_h_view),
                },
            ],
        });
        let bg_min_v = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dehaze-min-v-bg"),
            layout: &p.min_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: min_v_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&dn_h_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&dn_min_view),
                },
            ],
        });
        let bg_pack = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dehaze-pack-bg"),
            layout: &p.pack_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: pack_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&lo_src_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&dn_min_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&packed_view),
                },
            ],
        });
        let bg_box_h_pack = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dehaze-box-h-pack-bg"),
            layout: &p.box_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: box_h_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&packed_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&packed_h_view),
                },
            ],
        });
        let bg_box_v_pack = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dehaze-box-v-pack-bg"),
            layout: &p.box_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: box_v_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&packed_h_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&packed_v_view),
                },
            ],
        });
        let bg_ab = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dehaze-ab-bg"),
            layout: &p.ab_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: ab_uni.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&packed_v_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&ab_view),
                },
            ],
        });
        let bg_box_h_ab = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dehaze-box-h-ab-bg"),
            layout: &p.box_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: box_h_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&ab_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&ab_h_view),
                },
            ],
        });
        let bg_box_v_ab = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dehaze-box-v-ab-bg"),
            layout: &p.box_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: box_v_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&ab_h_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&ab_v_view),
                },
            ],
        });
        let bg_apply = device.create_bind_group(&BindGroupDescriptor {
            label: Some("dehaze-apply-bg"),
            layout: &p.apply_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: apply_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&src_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&ab_v_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(&p.linear_sampler),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&out_view),
                },
            ],
        });

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

    pub(super) fn run_presence(
        &self,
        src: &Texture,
        dims: (u32, u32),
        edits: &Edits,
    ) -> PipelineResult<Arc<Texture>> {
        let _span = tracing::debug_span!("gpu.run_presence", w = dims.0, h = dims.1).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (w, h) = dims;
        let edits = edits.clamped();

        let radii = presence_radii(w, h);
        let pyramid_levels = presence_pyramid_levels(w, h, radii);

        let pyramid = LumaPyramidPass::allocate_pyramid(&self.ctx, w, h, pyramid_levels);
        let adjusted = device.create_texture(&TextureDescriptor {
            label: Some("presence-adjusted"),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count(w, h),
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.ctx.linear_format,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let amts = presence_amounts(&edits);
        let amounts: [f32; 4] = [amts.texture, amts.clarity, 0.0, 0.0];
        let mip_sel = presence_mips(w, h, radii);
        let mips: [u32; 4] = [mip_sel.texture, mip_sel.clarity, 0, 0];

        let mut uniform_bytes = vec![0u8; PRESENCE_UNIFORM_SIZE as usize];
        uniform_bytes[0..4].copy_from_slice(&w.to_le_bytes());
        uniform_bytes[4..8].copy_from_slice(&h.to_le_bytes());
        for (i, a) in amounts.iter().enumerate() {
            let off = 16 + i * 4;
            uniform_bytes[off..off + 4].copy_from_slice(&a.to_le_bytes());
        }
        for (i, m) in mips.iter().enumerate() {
            let off = 32 + i * 4;
            uniform_bytes[off..off + 4].copy_from_slice(&m.to_le_bytes());
        }
        let uniform_buf =
            self.uniform_pool
                .acquire(device, queue, &uniform_bytes, "presence-uniform");

        let src_view_full = src.create_view(&TextureViewDescriptor::default());
        let pyramid_full_view = pyramid.create_view(&TextureViewDescriptor::default());
        let adjusted_mip0_view = adjusted.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let pyramid_level_views: Vec<wgpu::TextureView> = (0..pyramid_levels)
            .map(|level| {
                pyramid.create_view(&TextureViewDescriptor {
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        let extract_bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("luma-extract-bg"),
            layout: &self.passes.luma_pyramid.extract_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&src_view_full),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&pyramid_level_views[0]),
                },
            ],
        });
        let mipgen_binds: Vec<wgpu::BindGroup> = (1..pyramid_levels)
            .map(|level| {
                device.create_bind_group(&BindGroupDescriptor {
                    label: Some("pyramid-mipgen-bg"),
                    layout: &self.passes.mipgen.layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(
                                &pyramid_level_views[(level - 1) as usize],
                            ),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(
                                &pyramid_level_views[level as usize],
                            ),
                        },
                    ],
                })
            })
            .collect();
        let presence_bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("presence-bg"),
            layout: &self.passes.presence.adjust_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&src_view_full),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&pyramid_full_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&adjusted_mip0_view),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("presence-enc"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("luma-extract-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.passes.luma_pyramid.extract_pipeline);
            pass.set_bind_group(0, &extract_bind, &[]);
            pass.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
        }
        if !mipgen_binds.is_empty() {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("pyramid-mipgen-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.passes.mipgen.pipeline);
            let mut mw = w;
            let mut mh = h;
            for bg in &mipgen_binds {
                let dst_w = (mw / 2).max(1);
                let dst_h = (mh / 2).max(1);
                pass.set_bind_group(0, bg, &[]);
                pass.dispatch_workgroups(dst_w.div_ceil(16), dst_h.div_ceil(16), 1);
                mw = dst_w;
                mh = dst_h;
            }
        }
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("presence-adjust-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.passes.presence.adjust_pipeline);
            pass.set_bind_group(0, &presence_bind, &[]);
            pass.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
        }
        self.encode_mipgen(&mut encoder, &adjusted, w, h);
        queue.submit(Some(encoder.finish()));

        Ok(Arc::new(adjusted))
    }

    pub(super) fn run_wb_prepare(
        &self,
        cached: &CachedFrame,
        frame: &RawFrame,
        edits: &Edits,
        setup: &crate::dcp_pipeline::DcpSetup,
    ) -> PipelineResult<Arc<Texture>> {
        let key = wb_cache_key(
            frame,
            edits,
            (cached.width, cached.height),
            setup.cam_to_srgb,
        );
        if let Some(t) = self.wb_cache.lock().get(&key).cloned() {
            tracing::debug!(target: "gpu_cache", "wb_base cache hit");
            return Ok(t);
        }
        let _span = tracing::debug_span!("gpu.run_wb_prepare", w = cached.width, h = cached.height)
            .entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = cached.width;
        let h = cached.height;

        let ctx_op = OpContext {
            render: RenderContext {
                wb_coeffs: frame.wb_coeffs,
                cam_to_srgb: setup.cam_to_srgb,
                is_raw: frame.is_raw,
                preview_mode: crate::frame::PreviewMode::None,
                dcp: setup.resolved.clone(),
            },
            scratch: OpScratch::default(),
        };

        let pass = &self.passes.wb_prepare;
        let built = &pass.built;
        let registry = &self.passes.registry;
        let mut uniform_bytes = vec![0u8; built.uniform_size];
        write_header(
            &mut uniform_bytes,
            [w, h],
            [w, h],
            [0.0, 0.0, 1.0, 1.0],
            [0, 0, 0, 0],
            [0.0; 4],
            [0.0; 4],
            [0.0; 4],
            [0, 0, 0, 0],
        );
        let mut active_mask: [u32; 4] = [0; 4];
        for slot in &built.color_ops {
            let op = &registry.ops()[slot.op_index];
            if op.is_active(edits) {
                let word = (slot.active_bit / 32) as usize;
                let shift = slot.active_bit % 32;
                active_mask[word] |= 1u32 << shift;
            }
            let mut buf = vec![0.0f32; slot.vec4_count * 4];
            op.write_gpu_uniform(edits, &ctx_op, &mut buf);
            let off = slot.uniform_offset;
            let bytes = slot.vec4_count * 16;
            uniform_bytes[off..off + bytes].copy_from_slice(bytemuck::cast_slice(&buf));
        }
        write_active_mask(&mut uniform_bytes, active_mask);

        let uniform_buf =
            self.uniform_pool
                .acquire(device, queue, &uniform_bytes, "wb-prepare-uniform");

        let wb_base = device.create_texture(&TextureDescriptor {
            label: Some("wb-base"),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count(w, h),
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.ctx.linear_format,
            usage: TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let src_view = cached
            .texture
            .create_view(&TextureViewDescriptor::default());
        let dst_view = wb_base.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("wb-prepare-bg"),
            layout: &pass.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&src_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&dst_view),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("wb-prepare-enc"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("wb-prepare-pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&pass.pipeline);
            cpass.set_bind_group(0, &bind, &[]);
            cpass.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
        }
        self.encode_mipgen(&mut encoder, &wb_base, w, h);
        queue.submit(Some(encoder.finish()));

        let out = Arc::new(wb_base);
        self.wb_cache.lock().put(key, out.clone());
        Ok(out)
    }

    pub(super) fn run_sensor(
        &self,
        src: &Arc<CachedFrame>,
        edits: &Edits,
    ) -> PipelineResult<Arc<CachedFrame>> {
        use crate::gpu::passes::sensor::SensorParams;

        let _span = tracing::debug_span!("gpu.run_sensor", w = src.width, h = src.height).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = src.width;
        let h = src.height;
        let params = SensorParams::from_edits(&edits.lens, w, h);
        let uniform_buf =
            self.uniform_pool
                .acquire(device, queue, bytemuck::bytes_of(&params), "sensor-uniform");
        let dst = device.create_texture(&TextureDescriptor {
            label: Some("sensor-out"),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count(w, h),
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.ctx.linear_format,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let src_view = src.texture.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let dst_view = dst.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let pass = &self.passes.sensor;
        let bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sensor-bg"),
            layout: &pass.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&src_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&dst_view),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("sensor-enc"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("sensor-pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&pass.pipeline);
            cpass.set_bind_group(0, &bind, &[]);
            cpass.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
        }
        self.encode_mipgen(&mut encoder, &dst, w, h);
        queue.submit(Some(encoder.finish()));
        Ok(Arc::new(CachedFrame {
            texture: Arc::new(dst),
            width: w,
            height: h,
        }))
    }

    pub(super) fn build_luma_pyramid(
        &self,
        src: &Texture,
        dims: (u32, u32),
    ) -> PipelineResult<Arc<Texture>> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (w, h) = dims;
        let radii = presence_radii(w, h);
        let pyramid_levels = presence_pyramid_levels(w, h, radii);
        let pyramid = LumaPyramidPass::allocate_pyramid(&self.ctx, w, h, pyramid_levels);
        let src_view = src.create_view(&TextureViewDescriptor::default());
        let level_views: Vec<wgpu::TextureView> = (0..pyramid_levels)
            .map(|level| {
                pyramid.create_view(&TextureViewDescriptor {
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();
        let extract_bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("luma-extract-bg-shadows"),
            layout: &self.passes.luma_pyramid.extract_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&src_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&level_views[0]),
                },
            ],
        });
        let mipgen_binds: Vec<wgpu::BindGroup> = (1..pyramid_levels)
            .map(|level| {
                device.create_bind_group(&BindGroupDescriptor {
                    label: Some("pyramid-mipgen-bg-shadows"),
                    layout: &self.passes.mipgen.layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(
                                &level_views[(level - 1) as usize],
                            ),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(&level_views[level as usize]),
                        },
                    ],
                })
            })
            .collect();
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("shadows-pyramid-enc"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("luma-extract-shadows"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.passes.luma_pyramid.extract_pipeline);
            pass.set_bind_group(0, &extract_bind, &[]);
            pass.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
        }
        if !mipgen_binds.is_empty() {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("pyramid-mipgen-shadows"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.passes.mipgen.pipeline);
            let mut mw = w;
            let mut mh = h;
            for bg in &mipgen_binds {
                let dst_w = (mw / 2).max(1);
                let dst_h = (mh / 2).max(1);
                pass.set_bind_group(0, bg, &[]);
                pass.dispatch_workgroups(dst_w.div_ceil(16), dst_h.div_ceil(16), 1);
                mw = dst_w;
                mh = dst_h;
            }
        }
        queue.submit(Some(encoder.finish()));
        Ok(Arc::new(pyramid))
    }
}
