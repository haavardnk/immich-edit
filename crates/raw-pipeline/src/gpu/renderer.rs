use std::num::NonZeroUsize;
use std::sync::Arc;

use parking_lot::Mutex;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    AddressMode, BindGroupDescriptor, BindGroupEntry, BindingResource, BufferUsages,
    CommandEncoder, CommandEncoderDescriptor, ComputePassDescriptor, Extent3d, FilterMode,
    SamplerDescriptor, Texture, TextureDescriptor, TextureDimension, TextureUsages,
    TextureViewDescriptor,
};

use crate::edits::Edits;
use crate::encode::encode_from_rgba8;
use crate::frame::{RawFrame, RenderOptions, RenderedImage};
use crate::histogram::Histogram;
use crate::ops::{GpuOpKind, OpContext};
use crate::{PipelineError, PipelineResult};

use super::context::GpuContext;
use super::helpers::{DemosaicParams, cfa_to_indices, mip_count, scale_to_max};
use super::passes::GpuPasses;
use super::passes::luma_pyramid::LumaPyramidPass;
use super::passes::presence::PRESENCE_UNIFORM_SIZE;
use super::readback::{copy_texture_to_buffer, read_rgba8, read_rgba16f_as_rgb};
use super::resources::{OutputTargets, SharpenTargets};
use super::uniforms::{write_active_mask, write_header};
use crate::presence::{presence_amounts, presence_mips, presence_pyramid_levels, presence_radii};

const CACHE_ITEMS: usize = 2;

struct CachedFrame {
    texture: Arc<Texture>,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPlan {
    Fast,
    Presence,
}

impl RenderPlan {
    pub fn select(edits: &Edits) -> Self {
        let b = &edits.basic;
        if b.texture != 0.0 || b.clarity != 0.0 || b.dehaze != 0.0 {
            Self::Presence
        } else {
            Self::Fast
        }
    }
}

pub struct GpuRenderer {
    ctx: Arc<GpuContext>,
    passes: Arc<GpuPasses>,
    cache: Mutex<lru::LruCache<u64, Arc<CachedFrame>>>,
    output_pool: Mutex<Option<OutputTargets>>,
    sharpen_pool: Mutex<Option<SharpenTargets>>,
}

impl GpuRenderer {
    pub fn new() -> PipelineResult<Self> {
        let ctx = GpuContext::new()?;
        let passes = Arc::new(GpuPasses::new(&ctx));
        Ok(Self {
            ctx,
            passes,
            cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(CACHE_ITEMS).expect("nonzero"),
            )),
            output_pool: Mutex::new(None),
            sharpen_pool: Mutex::new(None),
        })
    }

    pub fn adapter_label(&self) -> String {
        self.ctx.adapter_label()
    }

    fn frame_key(frame: &RawFrame) -> u64 {
        let ptr = frame.data.as_ptr() as usize as u64;
        let dims = ((frame.width as u64) << 32) | (frame.height as u64);
        ptr ^ dims
    }

    fn get_or_demosaic(&self, frame: &RawFrame) -> PipelineResult<Arc<CachedFrame>> {
        let key = Self::frame_key(frame);
        if let Some(c) = self.cache.lock().get(&key).cloned() {
            return Ok(c);
        }
        let cached = if frame.cpp == 3 {
            self.upload_rgb_texture(frame)?
        } else {
            self.demosaic_to_texture(frame)?
        };
        self.cache.lock().put(key, cached.clone());
        Ok(cached)
    }

    fn upload_rgb_texture(&self, frame: &RawFrame) -> PipelineResult<Arc<CachedFrame>> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = frame.width as u32;
        let h = frame.height as u32;

        let rgba_f16: Vec<u16> = frame
            .data
            .chunks_exact(3)
            .flat_map(|rgb| {
                [
                    half::f16::from_f32(rgb[0]).to_bits(),
                    half::f16::from_f32(rgb[1]).to_bits(),
                    half::f16::from_f32(rgb[2]).to_bits(),
                    half::f16::from_f32(1.0).to_bits(),
                ]
            })
            .collect();

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("linear-uploaded"),
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
                | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            bytemuck::cast_slice(&rgba_f16),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(w * 8),
                rows_per_image: Some(h),
            },
            Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );

        self.encode_mipgen(&texture, w, h);

        Ok(Arc::new(CachedFrame {
            texture: Arc::new(texture),
            width: w,
            height: h,
        }))
    }

    fn demosaic_to_texture(&self, frame: &RawFrame) -> PipelineResult<Arc<CachedFrame>> {
        if frame.cpp != 1 {
            return Err(PipelineError::Unsupported(
                "gpu demosaic requires single-plane bayer frame".into(),
            ));
        }
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = frame.width as u32;
        let h = frame.height as u32;

        let cfa = cfa_to_indices(&frame.cfa_pattern);
        let params = DemosaicParams {
            size: [w, h],
            _pad: [0, 0],
            cfa,
        };

        let uniform_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("demosaic-uniform"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let raw_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("raw-storage"),
            contents: bytemuck::cast_slice(&frame.data),
            usage: BufferUsages::STORAGE,
        });

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("linear-cached"),
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
        let view = texture.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("demosaic-bg"),
            layout: &self.passes.demosaic.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: raw_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&view),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("demosaic-enc"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("demosaic-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.passes.demosaic.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            let gx = w.div_ceil(16);
            let gy = h.div_ceil(16);
            pass.dispatch_workgroups(gx, gy, 1);
        }
        queue.submit(Some(encoder.finish()));
        self.encode_mipgen(&texture, w, h);

        Ok(Arc::new(CachedFrame {
            texture: Arc::new(texture),
            width: w,
            height: h,
        }))
    }

    fn encode_mipgen(&self, texture: &Texture, w: u32, h: u32) {
        let levels = mip_count(w, h);
        if levels <= 1 {
            return;
        }
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("mipgen-enc"),
        });
        let mut mip_w = w;
        let mut mip_h = h;
        for level in 1..levels {
            let src_view = texture.create_view(&TextureViewDescriptor {
                base_mip_level: level - 1,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let dst_w = (mip_w / 2).max(1);
            let dst_h = (mip_h / 2).max(1);
            let dst_view = texture.create_view(&TextureViewDescriptor {
                base_mip_level: level,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let bind = device.create_bind_group(&BindGroupDescriptor {
                label: Some("mipgen-bg"),
                layout: &self.passes.mipgen.layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&src_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&dst_view),
                    },
                ],
            });
            {
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("mipgen-pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.passes.mipgen.pipeline);
                pass.set_bind_group(0, &bind, &[]);
                pass.dispatch_workgroups(dst_w.div_ceil(16), dst_h.div_ceil(16), 1);
            }
            mip_w = dst_w;
            mip_h = dst_h;
        }
        queue.submit(Some(encoder.finish()));
    }

    fn process(
        &self,
        pass: &super::passes::process::ProcessFastPass,
        src_texture: &Texture,
        src_dims: (u32, u32),
        frame: &RawFrame,
        edits: &Edits,
        opts: &RenderOptions,
    ) -> PipelineResult<RenderedImage> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;

        let edits = edits.clamped();
        let sharpen_active = edits.detail.sharpen_active();

        if edits.detail.luma_nr_active() || edits.detail.color_nr_active() {
            return Err(PipelineError::Unsupported(
                "gpu pipeline: noise reduction not implemented".into(),
            ));
        }

        for op in self.passes.registry.active(&edits) {
            if op.gpu_kind() == GpuOpKind::Presence {
                continue;
            }
            if op.gpu_kind() == GpuOpKind::Detail {
                continue;
            }
            if op.gpu().is_none() {
                return Err(PipelineError::Unsupported(format!(
                    "gpu pipeline missing op: {}",
                    op.id()
                )));
            }
        }

        let (sensor_w, sensor_h) = src_dims;
        let (display_w, display_h) = if frame.orientation.0 {
            (sensor_h, sensor_w)
        } else {
            (sensor_w, sensor_h)
        };

        let (oriented_w, oriented_h) = match edits.geometry.rotate {
            90 | 270 => (display_h, display_w),
            _ => (display_w, display_h),
        };

        let crop = edits
            .geometry
            .crop
            .unwrap_or(crate::edits::CropRect::full());
        let angle = edits.geometry.rotate_angle;
        let bbox = crate::geom::rotated_bbox(oriented_w as f32, oriented_h as f32, angle);
        let bw = bbox.w;
        let bh = bbox.h;
        let crop_w_px = (crop.w * bw).round().max(1.0) as u32;
        let crop_h_px = (crop.h * bh).round().max(1.0) as u32;
        let (out_w, out_h) = scale_to_max(crop_w_px, crop_h_px, opts.max_edge);

        let src_max = sensor_w.max(sensor_h) as f32;
        let out_max = out_w.max(out_h) as f32;
        let lod = if src_max > out_max {
            (src_max / out_max).log2()
        } else {
            0.0
        };

        let a_rad = crate::geom::deg_to_rad(angle);
        let cos_a = a_rad.cos();
        let sin_a = a_rad.sin();

        let (ot, oh_h, oh_v) = frame.orientation;
        let orient_packed = (oh_h as u32) | ((oh_v as u32) << 1) | ((ot as u32) << 2);

        let xyz_to_cam = if frame.color_matrices.len() >= 2 {
            let cct = crate::color::estimate_scene_cct(
                frame.wb_coeffs,
                &frame.color_matrices.last().unwrap().1,
            );
            crate::color::interpolate_xyz_to_cam(&frame.color_matrices, cct)
        } else {
            frame.xyz_to_cam
        };
        let cam_to_srgb = if frame.is_raw && !crate::color::is_unusable_matrix(&xyz_to_cam) {
            crate::color::cam_to_srgb_matrix(xyz_to_cam)
        } else {
            crate::color::identity_3x3()
        };
        let ctx_op = OpContext {
            wb_coeffs: frame.wb_coeffs,
            cam_to_srgb,
            is_raw: frame.is_raw,
            preview_mode: opts.preview_mode,
        };
        let built = &pass.built;
        let registry = &self.passes.registry;
        let mut uniform_bytes = vec![0u8; built.uniform_size];
        write_header(
            &mut uniform_bytes,
            [sensor_w, sensor_h],
            [out_w, out_h],
            [crop.x, crop.y, crop.w, crop.h],
            [
                edits.geometry.rotate as u32,
                edits.geometry.flip_h as u32,
                edits.geometry.flip_v as u32,
                orient_packed,
            ],
            [lod, 0.0, 0.0, 0.0],
            [cos_a, sin_a, bw, bh],
            [oriented_w as f32, oriented_h as f32, 0.0, 0.0],
        );
        let mut active_mask: [u32; 4] = [0; 4];
        for slot in &built.color_ops {
            let op = &registry.ops()[slot.op_index];
            if op.is_active(&edits) {
                let word = (slot.active_bit / 32) as usize;
                let shift = slot.active_bit % 32;
                active_mask[word] |= 1u32 << shift;
            }
            let mut buf = vec![0.0f32; slot.vec4_count * 4];
            op.write_gpu_uniform(&edits, &ctx_op, &mut buf);
            let off = slot.uniform_offset;
            let bytes = slot.vec4_count * 16;
            uniform_bytes[off..off + bytes].copy_from_slice(bytemuck::cast_slice(&buf));
        }
        write_active_mask(&mut uniform_bytes, active_mask);

        let uniform_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("process-uniform"),
            contents: &uniform_bytes,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let src_view = src_texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("linear-samp"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        let mut pool = self.output_pool.lock();
        let fits = pool.as_ref().is_some_and(|p| p.fits(out_w, out_h));
        if !fits {
            *pool = Some(OutputTargets::allocate(&self.ctx, out_w, out_h));
        }
        let p = pool.as_ref().expect("pool populated");
        let out_view = p.texture.create_view(&TextureViewDescriptor::default());
        let linear_view = p
            .linear_texture
            .create_view(&TextureViewDescriptor::default());

        let bind = device.create_bind_group(&BindGroupDescriptor {
            label: Some("process-bg"),
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
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&out_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&linear_view),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("process-enc"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("process-pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&pass.pipeline);
            cpass.set_bind_group(0, &bind, &[]);
            let gx = out_w.div_ceil(16);
            let gy = out_h.div_ceil(16);
            cpass.dispatch_workgroups(gx, gy, 1);
        }

        let sharpen_preview = matches!(
            opts.preview_mode,
            crate::frame::PreviewMode::SharpenMask
                | crate::frame::PreviewMode::SharpenRadius
                | crate::frame::PreviewMode::SharpenDetail
        );
        let sharpen_pool_guard = if sharpen_active || sharpen_preview {
            let mut spool = self.sharpen_pool.lock();
            let sfits = spool.as_ref().is_some_and(|s| s.fits(out_w, out_h));
            if !sfits {
                *spool = Some(SharpenTargets::allocate(&self.ctx, out_w, out_h));
            }
            Some(spool)
        } else {
            None
        };
        if let Some(spool) = sharpen_pool_guard.as_ref() {
            let s = spool.as_ref().expect("sharpen pool populated");
            self.encode_sharpen(
                &mut encoder,
                &edits.detail,
                p,
                s,
                out_w,
                out_h,
                opts.preview_mode,
            );
        }

        copy_texture_to_buffer(&mut encoder, &p.texture, &p.readback, out_w, out_h);
        copy_texture_to_buffer(
            &mut encoder,
            &p.linear_texture,
            &p.linear_readback,
            out_w,
            out_h,
        );
        queue.submit(Some(encoder.finish()));

        let rgba = read_rgba8(&self.ctx, &p.readback, out_w, out_h)?;
        let linear_rgb = read_rgba16f_as_rgb(&self.ctx, &p.linear_readback, out_w, out_h)?;
        drop(pool);

        let ((histogram, linear_histogram), bytes) = rayon::join(
            || {
                rayon::join(
                    || Histogram::from_rgba8(&rgba),
                    || Histogram::from_rgb(&linear_rgb, out_w as usize, out_h as usize),
                )
            },
            || encode_from_rgba8(&rgba, out_w, out_h, &opts.output),
        );
        let bytes = bytes?;

        Ok(RenderedImage {
            bytes,
            histogram,
            linear_histogram: Some(linear_histogram),
            width: out_w,
            height: out_h,
            source_w: oriented_w,
            source_h: oriented_h,
            renderer: "gpu".into(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_sharpen(
        &self,
        encoder: &mut CommandEncoder,
        d: &crate::edits::DetailEdits,
        out: &OutputTargets,
        sh: &SharpenTargets,
        w: u32,
        h: u32,
        preview: crate::frame::PreviewMode,
    ) {
        let device = &self.ctx.device;
        let sigma = (d.sharpen_radius as f32).max(0.01);
        let radius = (sigma * 3.0).ceil();
        let amount = d.sharpen_amount as f32;
        let detail_weight = 0.5 + 0.5 * (d.sharpen_detail as f32 / 100.0);
        let masking = (d.sharpen_masking as f32 / 100.0).clamp(0.0, 1.0);
        let preview_mode_u: u32 = match preview {
            crate::frame::PreviewMode::None => 0,
            crate::frame::PreviewMode::SharpenMask => 1,
            crate::frame::PreviewMode::SharpenRadius => 2,
            crate::frame::PreviewMode::SharpenDetail => 3,
        };
        let use_mask = if masking > 0.0 || preview_mode_u == 1 {
            1u32
        } else {
            0u32
        };
        let masking_thresh = masking * 0.5;
        let masking_softness = 0.1f32;

        let blur_uniform = |axis: u32| -> wgpu::Buffer {
            let mut bytes = [0u8; 32];
            bytes[0..4].copy_from_slice(&sigma.to_ne_bytes());
            bytes[4..8].copy_from_slice(&radius.to_ne_bytes());
            bytes[8..12].copy_from_slice(&w.to_ne_bytes());
            bytes[12..16].copy_from_slice(&h.to_ne_bytes());
            bytes[16..20].copy_from_slice(&axis.to_ne_bytes());
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("sharpen-blur-uniform"),
                contents: &bytes,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let linear_view = out
            .linear_texture
            .create_view(&TextureViewDescriptor::default());
        let blur_h_view = sh.blur_h.create_view(&TextureViewDescriptor::default());
        let blur_full_view = sh.blur_full.create_view(&TextureViewDescriptor::default());
        let out_view = out.texture.create_view(&TextureViewDescriptor::default());

        let pass_h = &self.passes.output_sharpen;
        let ub_h = blur_uniform(0);
        let bg_h = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sharpen-blur-h-bg"),
            layout: &pass_h.blur_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: ub_h.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&linear_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&blur_h_view),
                },
            ],
        });
        let ub_v = blur_uniform(1);
        let bg_v = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sharpen-blur-v-bg"),
            layout: &pass_h.blur_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: ub_v.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&blur_h_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&blur_full_view),
                },
            ],
        });

        let mut combine_bytes = [0u8; 32];
        combine_bytes[0..4].copy_from_slice(&amount.to_ne_bytes());
        combine_bytes[4..8].copy_from_slice(&detail_weight.to_ne_bytes());
        combine_bytes[8..12].copy_from_slice(&masking_thresh.to_ne_bytes());
        combine_bytes[12..16].copy_from_slice(&masking_softness.to_ne_bytes());
        combine_bytes[16..20].copy_from_slice(&w.to_ne_bytes());
        combine_bytes[20..24].copy_from_slice(&h.to_ne_bytes());
        combine_bytes[24..28].copy_from_slice(&use_mask.to_ne_bytes());
        combine_bytes[28..32].copy_from_slice(&preview_mode_u.to_ne_bytes());
        let ub_c = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sharpen-combine-uniform"),
            contents: &combine_bytes,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bg_c = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sharpen-combine-bg"),
            layout: &pass_h.combine_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: ub_c.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&linear_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&blur_full_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&out_view),
                },
            ],
        });

        let gx = w.div_ceil(16);
        let gy = h.div_ceil(16);
        {
            let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("sharpen-blur-h"),
                timestamp_writes: None,
            });
            cp.set_pipeline(&pass_h.blur_pipeline);
            cp.set_bind_group(0, &bg_h, &[]);
            cp.dispatch_workgroups(gx, gy, 1);
        }
        {
            let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("sharpen-blur-v"),
                timestamp_writes: None,
            });
            cp.set_pipeline(&pass_h.blur_pipeline);
            cp.set_bind_group(0, &bg_v, &[]);
            cp.dispatch_workgroups(gx, gy, 1);
        }
        {
            let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("sharpen-combine"),
                timestamp_writes: None,
            });
            cp.set_pipeline(&pass_h.combine_pipeline);
            cp.set_bind_group(0, &bg_c, &[]);
            cp.dispatch_workgroups(gx, gy, 1);
        }
    }

    pub fn render(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
    ) -> PipelineResult<RenderedImage> {
        self.render_with_cancel(frame, edits, options, None)
    }

    fn run_presence(
        &self,
        src: &Texture,
        dims: (u32, u32),
        edits: &Edits,
    ) -> PipelineResult<Arc<Texture>> {
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
        let amounts: [f32; 4] = [amts.texture, amts.clarity, amts.dehaze, 0.0];
        let mip_sel = presence_mips(w, h, radii);
        let mips: [u32; 4] = [mip_sel.texture, mip_sel.clarity, mip_sel.dehaze, 0];

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
        let uniform_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("presence-uniform"),
            contents: &uniform_bytes,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

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
        queue.submit(Some(encoder.finish()));

        self.encode_mipgen(&adjusted, w, h);

        Ok(Arc::new(adjusted))
    }

    fn run_wb_prepare(
        &self,
        cached: &CachedFrame,
        frame: &RawFrame,
        edits: &Edits,
    ) -> PipelineResult<Arc<Texture>> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = cached.width;
        let h = cached.height;

        let xyz_to_cam = if frame.color_matrices.len() >= 2 {
            let cct = crate::color::estimate_scene_cct(
                frame.wb_coeffs,
                &frame.color_matrices.last().unwrap().1,
            );
            crate::color::interpolate_xyz_to_cam(&frame.color_matrices, cct)
        } else {
            frame.xyz_to_cam
        };
        let cam_to_srgb = if frame.is_raw && !crate::color::is_unusable_matrix(&xyz_to_cam) {
            crate::color::cam_to_srgb_matrix(xyz_to_cam)
        } else {
            crate::color::identity_3x3()
        };
        let ctx_op = OpContext {
            wb_coeffs: frame.wb_coeffs,
            cam_to_srgb,
            is_raw: frame.is_raw,
            preview_mode: crate::frame::PreviewMode::None,
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

        let uniform_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("wb-prepare-uniform"),
            contents: &uniform_bytes,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

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
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
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
        queue.submit(Some(encoder.finish()));

        self.encode_mipgen(&wb_base, w, h);

        Ok(Arc::new(wb_base))
    }

    pub fn render_with_cancel(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<RenderedImage> {
        crate::cancel::check(cancel)?;
        let plan = RenderPlan::select(edits);
        let cached = self.get_or_demosaic(frame)?;
        crate::cancel::check(cancel)?;
        let dims = (cached.width, cached.height);
        let edits_c = edits.clamped();
        match plan {
            RenderPlan::Fast => {
                let out = self.process(
                    &self.passes.process_fast,
                    cached.texture.as_ref(),
                    dims,
                    frame,
                    edits,
                    options,
                )?;
                crate::cancel::check(cancel)?;
                Ok(out)
            }
            RenderPlan::Presence => {
                let wb_base = self.run_wb_prepare(&cached, frame, &edits_c)?;
                crate::cancel::check(cancel)?;
                let adjusted = self.run_presence(&wb_base, dims, &edits_c)?;
                crate::cancel::check(cancel)?;
                let out = self.process(
                    &self.passes.process_post_wb,
                    &adjusted,
                    dims,
                    frame,
                    edits,
                    options,
                )?;
                crate::cancel::check(cancel)?;
                Ok(out)
            }
        }
    }
}
