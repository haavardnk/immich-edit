use std::sync::Arc;

use wgpu::{
    BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor, Extent3d, Texture,
    TextureDescriptor, TextureDimension, TextureUsages, TextureViewDescriptor,
};

use crate::PipelineResult;
use crate::edits::Edits;
use crate::frame::RawFrame;
use crate::gpu::dispatch::{bind_group, dispatch_2d, samp, tex};
use crate::gpu::passes::capture_sharpen as capture;
use crate::gpu::passes::capture_sharpen::{
    CAPTURE_KERNEL_MAX, CaptureApplyParams, CaptureBlurParams, CaptureLumaParams,
};
use crate::gpu::passes::dehaze::{
    DehazeApplyParams, DehazeDownsampleParams, DehazeFilterParams, DehazeNormParams,
};
use crate::gpu::passes::luma_pyramid::LumaPyramidPass;
use crate::gpu::passes::nr::NrParams;
use crate::gpu::passes::nr_smooth::NrSmoothParams;
use crate::gpu::passes::presence::PresenceParams;
use crate::gpu::texture_pool::TextureKey;
use crate::presence::{presence_amounts, presence_mips, presence_pyramid_levels, presence_radii};

use crate::gpu::helpers::mip_count;
use crate::gpu::uniforms::{ProcessHeader, write_active_mask, write_header};
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

    pub(super) fn run_capture_sharpen(
        &self,
        src: &Texture,
        dims: (u32, u32),
        sigma: f32,
        key: u64,
    ) -> PipelineResult<Arc<Texture>> {
        if let Some(t) = self.capture_cache.lock().get(&key).cloned() {
            tracing::debug!(target: "gpu_cache", "capture_sharpen cache hit");
            return Ok(t);
        }
        let _span =
            tracing::debug_span!("gpu.run_capture_sharpen", w = dims.0, h = dims.1).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (w, h) = dims;
        let p = &self.passes.capture_sharpen;
        let kernel = crate::ops::capture_sharpen::gaussian_kernel(sigma);
        let radius = (kernel.len() / 2) as u32;

        let luma_params = CaptureLumaParams {
            size: [w, h],
            _pad: [0; 2],
        };
        let luma_buf = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&luma_params),
            "capture-luma-u",
        );

        let make_blur_u = |axis: u32, mode: u32, label: &'static str| {
            let mut params = CaptureBlurParams {
                size: [w, h],
                radius,
                axis,
                mode,
                _pad: [0; 3],
                kernel: [0.0; CAPTURE_KERNEL_MAX],
            };
            params.kernel[..kernel.len()].copy_from_slice(&kernel);
            self.uniform_pool
                .acquire(device, queue, bytemuck::bytes_of(&params), label)
        };
        let blur_h_buf = make_blur_u(0, 0, "capture-blur-h-u");
        let blur_ratio_buf = make_blur_u(1, 1, "capture-blur-ratio-u");
        let blur_mul_buf = make_blur_u(1, 2, "capture-blur-mul-u");

        let apply_params = CaptureApplyParams {
            size: [w, h],
            radius,
            _pad: 0,
        };
        let apply_buf = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&apply_params),
            "capture-apply-u",
        );

        let scratch_key = TextureKey::new(
            capture::CAPTURE_SCRATCH_FORMAT,
            w,
            h,
            1,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        );
        let luma = self
            .texture_pool
            .acquire(device, scratch_key, "capture-luma");
        let est_a = self
            .texture_pool
            .acquire(device, scratch_key, "capture-est-a");
        let est_b = self
            .texture_pool
            .acquire(device, scratch_key, "capture-est-b");
        let tmp = self
            .texture_pool
            .acquire(device, scratch_key, "capture-tmp");

        let out = device.create_texture(&TextureDescriptor {
            label: Some("capture-sharpen-out"),
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
        let luma_view = luma.create_view(&TextureViewDescriptor::default());
        let est_a_view = est_a.create_view(&TextureViewDescriptor::default());
        let est_b_view = est_b.create_view(&TextureViewDescriptor::default());
        let tmp_view = tmp.create_view(&TextureViewDescriptor::default());
        let out_view = out.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let bg_luma = bind_group(
            device,
            "capture-luma-bg",
            &p.luma_layout,
            &[
                luma_buf.as_entire_binding(),
                tex(&src_view),
                tex(&luma_view),
                tex(&est_a_view),
            ],
        );
        let make_blur_bg = |uniform: &crate::gpu::uniform_pool::PooledUniform,
                            read: &wgpu::TextureView,
                            aux: &wgpu::TextureView,
                            write: &wgpu::TextureView| {
            bind_group(
                device,
                "capture-blur-bg",
                &p.blur_layout,
                &[uniform.as_entire_binding(), tex(read), tex(aux), tex(write)],
            )
        };
        let steps: Vec<[wgpu::BindGroup; 4]> =
            [(&est_a_view, &est_b_view), (&est_b_view, &est_a_view)]
                .iter()
                .map(|(src_est, dst_est)| {
                    [
                        make_blur_bg(&blur_h_buf, src_est, &luma_view, &tmp_view),
                        make_blur_bg(&blur_ratio_buf, &tmp_view, &luma_view, dst_est),
                        make_blur_bg(&blur_h_buf, dst_est, &luma_view, &tmp_view),
                        make_blur_bg(&blur_mul_buf, &tmp_view, src_est, dst_est),
                    ]
                })
                .collect();
        let bg_apply = bind_group(
            device,
            "capture-apply-bg",
            &p.apply_layout,
            &[
                apply_buf.as_entire_binding(),
                tex(&src_view),
                tex(&luma_view),
                tex(&est_a_view),
                tex(&out_view),
            ],
        );

        let gx = w.div_ceil(16);
        let gy = h.div_ceil(16);
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("capture-sharpen-enc"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("capture-sharpen-pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&p.luma_pipeline);
            cpass.set_bind_group(0, &bg_luma, &[]);
            cpass.dispatch_workgroups(gx, gy, 1);
            cpass.set_pipeline(&p.blur_pipeline);
            for i in 0..crate::ops::capture_sharpen::ITERATIONS {
                for bg in steps[i % 2].iter() {
                    cpass.set_bind_group(0, bg, &[]);
                    cpass.dispatch_workgroups(gx, gy, 1);
                }
            }
            cpass.set_pipeline(&p.apply_pipeline);
            cpass.set_bind_group(0, &bg_apply, &[]);
            cpass.dispatch_workgroups(gx, gy, 1);
        }
        self.encode_mipgen(&mut encoder, &out, w, h);
        queue.submit(Some(encoder.finish()));

        let out = Arc::new(out);
        self.capture_cache.lock().put(key, out.clone());
        Ok(out)
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

        Ok(crate::cpu::dehaze::atmosphere_from_rgb(
            &rgb,
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
        let moment_key = TextureKey::new(
            crate::gpu::passes::dehaze::MOMENT_FORMAT,
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

        let params = PresenceParams {
            size: [w, h],
            _pad0: [0; 2],
            amounts,
            mips,
        };
        let uniform_buf = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&params),
            "presence-uniform",
        );

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

        let extract_bind = bind_group(
            device,
            "luma-extract-bg",
            &self.passes.luma_pyramid.extract_layout,
            &[tex(&src_view_full), tex(&pyramid_level_views[0])],
        );
        let mipgen_binds: Vec<wgpu::BindGroup> = (1..pyramid_levels)
            .map(|level| {
                bind_group(
                    device,
                    "pyramid-mipgen-bg",
                    &self.passes.mipgen.layout,
                    &[
                        tex(&pyramid_level_views[(level - 1) as usize]),
                        tex(&pyramid_level_views[level as usize]),
                    ],
                )
            })
            .collect();
        let presence_bind = bind_group(
            device,
            "presence-bg",
            &self.passes.presence.adjust_layout,
            &[
                uniform_buf.as_entire_binding(),
                tex(&src_view_full),
                tex(&pyramid_full_view),
                tex(&adjusted_mip0_view),
            ],
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("presence-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "luma-extract-pass",
            &self.passes.luma_pyramid.extract_pipeline,
            &extract_bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
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
        dispatch_2d(
            &mut encoder,
            "presence-adjust-pass",
            &self.passes.presence.adjust_pipeline,
            &presence_bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
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
                capture_sigma: frame.capture_sigma,
                preview_mode: crate::frame::PreviewMode::None,
                roi: None,
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
            &ProcessHeader {
                src_size: [w, h],
                out_size: [w, h],
                crop: [0.0, 0.0, 1.0, 1.0],
                flags: [0, 0, 0, 0],
                geom_extra: [0.0; 4],
                active_mask: [0; 4],
                geom_extra2: [0.0; 4],
                geom_extra3: [0.0; 4],
                output: [0, 0, 0, 0],
                perspective: crate::perspective::IDENTITY_ROWS,
            },
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
        let bind = bind_group(
            device,
            "wb-prepare-bg",
            &pass.layout,
            &[
                uniform_buf.as_entire_binding(),
                tex(&src_view),
                tex(&dst_view),
            ],
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("wb-prepare-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "wb-prepare-pass",
            &pass.pipeline,
            &bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
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
        let bind = bind_group(
            device,
            "sensor-bg",
            &pass.layout,
            &[
                uniform_buf.as_entire_binding(),
                tex(&src_view),
                tex(&dst_view),
            ],
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("sensor-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "sensor-pass",
            &pass.pipeline,
            &bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
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
        let extract_bind = bind_group(
            device,
            "luma-extract-bg-shadows",
            &self.passes.luma_pyramid.extract_layout,
            &[tex(&src_view), tex(&level_views[0])],
        );
        let mipgen_binds: Vec<wgpu::BindGroup> = (1..pyramid_levels)
            .map(|level| {
                bind_group(
                    device,
                    "pyramid-mipgen-bg-shadows",
                    &self.passes.mipgen.layout,
                    &[
                        tex(&level_views[(level - 1) as usize]),
                        tex(&level_views[level as usize]),
                    ],
                )
            })
            .collect();
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("shadows-pyramid-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "luma-extract-shadows",
            &self.passes.luma_pyramid.extract_pipeline,
            &extract_bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
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
