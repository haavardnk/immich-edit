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
use crate::ops::{GpuOpKind, OpContext, OpScratch, RenderContext};
use crate::{PipelineError, PipelineResult};

use super::context::GpuContext;
use super::helpers::{DemosaicParams, cfa_to_indices, mip_count, scale_to_max};
use super::passes::GpuPasses;
use super::passes::luma_pyramid::LumaPyramidPass;
use super::passes::presence::PRESENCE_UNIFORM_SIZE;
use super::readback::{copy_texture_to_buffer, read_rgba8, read_rgba16f_as_rgb};
use super::resources::{OutputTargets, SharpenTargets};
use super::texture_pool::{TextureKey, TexturePool};
use super::uniform_pool::UniformPool;
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
        let d = &edits.detail;
        if b.texture != 0.0
            || b.clarity != 0.0
            || b.dehaze != 0.0
            || edits.tone.shadows != 0.0
            || d.luma_nr_active()
            || d.color_nr_active()
        {
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
    atm_cache: Mutex<lru::LruCache<u64, [f32; 3]>>,
    texture_pool: Arc<TexturePool>,
    uniform_pool: Arc<UniformPool>,
    output_pool: Mutex<Option<OutputTargets>>,
    sharpen_pool: Mutex<Option<SharpenTargets>>,
}

const ATM_CACHE_ITEMS: usize = 16;
const TEXTURE_POOL_CAP_PER_KEY: usize = 4;
const UNIFORM_POOL_CAP_PER_SIZE: usize = 8;

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
            atm_cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(ATM_CACHE_ITEMS).expect("nonzero"),
            )),
            texture_pool: TexturePool::new(TEXTURE_POOL_CAP_PER_KEY),
            uniform_pool: UniformPool::new(UNIFORM_POOL_CAP_PER_SIZE),
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
        let _span = tracing::debug_span!(
            "gpu.upload_rgb",
            w = frame.width as u32,
            h = frame.height as u32
        )
        .entered();
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

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("upload-mipgen-enc"),
        });
        self.encode_mipgen(&mut encoder, &texture, w, h);
        queue.submit(Some(encoder.finish()));

        Ok(Arc::new(CachedFrame {
            texture: Arc::new(texture),
            width: w,
            height: h,
        }))
    }

    fn demosaic_to_texture(&self, frame: &RawFrame) -> PipelineResult<Arc<CachedFrame>> {
        let _span = tracing::debug_span!(
            "gpu.demosaic",
            w = frame.width as u32,
            h = frame.height as u32
        )
        .entered();
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

        let uniform_buf = self.uniform_pool.acquire(device, queue, bytemuck::bytes_of(&params), "demosaic-uniform");

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
        self.encode_mipgen(&mut encoder, &texture, w, h);
        queue.submit(Some(encoder.finish()));

        Ok(Arc::new(CachedFrame {
            texture: Arc::new(texture),
            width: w,
            height: h,
        }))
    }

    fn encode_mipgen(&self, encoder: &mut wgpu::CommandEncoder, texture: &Texture, w: u32, h: u32) {
        let _span = tracing::trace_span!("gpu.mipgen", w = w, h = h).entered();
        let levels = mip_count(w, h);
        if levels <= 1 {
            return;
        }
        let device = &self.ctx.device;
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
    }

    #[allow(clippy::too_many_arguments)]
    fn process(
        &self,
        pass: &super::passes::process::ProcessFastPass,
        src_texture: &Texture,
        src_dims: (u32, u32),
        frame: &RawFrame,
        edits: &Edits,
        opts: &RenderOptions,
        shadows_blur: Option<&wgpu::TextureView>,
    ) -> PipelineResult<RenderedImage> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;

        let edits = edits.clamped();
        let sharpen_active = edits.detail.sharpen_active();
        let effects_active = edits.effects.any_active();

        for op in self.passes.registry.active(&edits) {
            if op.gpu_kind() == GpuOpKind::Presence {
                continue;
            }
            if op.gpu_kind() == GpuOpKind::Detail {
                continue;
            }
            if op.id() == "dehaze" {
                continue;
            }
            if op.stage() == crate::ops::Stage::Output {
                continue;
            }
            if op.stage() == crate::ops::Stage::Sensor {
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
            render: RenderContext {
                wb_coeffs: frame.wb_coeffs,
                cam_to_srgb,
                is_raw: frame.is_raw,
                preview_mode: opts.preview_mode.clone(),
            },
            scratch: OpScratch { shadows_blur: None },
        };
        let built = &pass.built;
        let registry = &self.passes.registry;
        let shadows_mip_f = {
            let radii = presence_radii(src_dims.0, src_dims.1);
            let mips = presence_mips(src_dims.0, src_dims.1, radii);
            mips.shadows as f32
        };
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
            [lod, shadows_mip_f, 0.0, 0.0],
            [cos_a, sin_a, bw, bh],
            [oriented_w as f32, oriented_h as f32, 0.0, 0.0],
            [
                crate::tone::tonemap_kind_index(edits.output.tonemap),
                0,
                0,
                0,
            ],
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

        let uniform_buf = self.uniform_pool.acquire(device, queue, &uniform_bytes, "process-uniform");

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

        let dummy_shadows = if shadows_blur.is_none() {
            Some(make_dummy_luma(&self.ctx))
        } else {
            None
        };
        let dummy_view = dummy_shadows
            .as_ref()
            .map(|t| t.create_view(&TextureViewDescriptor::default()));
        let shadows_view_ref: &wgpu::TextureView =
            shadows_blur.unwrap_or_else(|| dummy_view.as_ref().unwrap());

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
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(shadows_view_ref),
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

        let effective_layers: Vec<&crate::edits::MaskLayer> =
            edits.masks.iter().filter(|l| l.is_effective()).collect();
        let has_masks = !effective_layers.is_empty();
        let mut accum_in_alt = false;
        let mut _retained_bufs: Vec<wgpu::Buffer> = Vec::new();
        let mut _retained_uniforms: Vec<super::uniform_pool::PooledUniform> = Vec::new();
        let mut _retained_binds: Vec<wgpu::BindGroup> = Vec::new();
        if has_masks {
            let scratch_linear_view = p
                .mask_scratch_linear
                .create_view(&TextureViewDescriptor::default());
            let scratch_tone_view = p
                .mask_scratch_tone
                .create_view(&TextureViewDescriptor::default());
            let weight_view = p.mask_weight.create_view(&TextureViewDescriptor::default());
            let accum_alt_view = p
                .mask_accum_alt
                .create_view(&TextureViewDescriptor::default());
            let linear_view2 = p
                .linear_texture
                .create_view(&TextureViewDescriptor::default());

            let mut slot_map: std::collections::HashMap<String, u32> =
                std::collections::HashMap::new();
            for layer in &effective_layers {
                for comp in &layer.components {
                    if !comp.enabled {
                        continue;
                    }
                    let crate::edits::MaskComponentKind::Brush { raster_id } = &comp.kind else {
                        continue;
                    };
                    if slot_map.len() as u32 >= crate::gpu::passes::mask_weight::ATLAS_LAYERS {
                        break;
                    }
                    if !slot_map.contains_key(raster_id) && opts.rasters.contains_key(raster_id) {
                        let slot = slot_map.len() as u32;
                        slot_map.insert(raster_id.clone(), slot);
                    }
                }
            }
            let atlas = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("mask-raster-atlas"),
                size: Extent3d {
                    width: crate::gpu::passes::mask_weight::ATLAS_DIM,
                    height: crate::gpu::passes::mask_weight::ATLAS_DIM,
                    depth_or_array_layers: crate::gpu::passes::mask_weight::ATLAS_LAYERS,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            for (raster_id, slot) in &slot_map {
                let Some(raster) = opts.rasters.get(raster_id) else {
                    continue;
                };
                let bytes = crate::gpu::passes::mask_weight::resample_raster_to_atlas(raster);
                self.ctx.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &atlas,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: *slot,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &bytes,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(crate::gpu::passes::mask_weight::ATLAS_DIM),
                        rows_per_image: Some(crate::gpu::passes::mask_weight::ATLAS_DIM),
                    },
                    Extent3d {
                        width: crate::gpu::passes::mask_weight::ATLAS_DIM,
                        height: crate::gpu::passes::mask_weight::ATLAS_DIM,
                        depth_or_array_layers: 1,
                    },
                );
            }
            let atlas_view = atlas.create_view(&wgpu::TextureViewDescriptor {
                label: Some("mask-raster-atlas-view"),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });
            let atlas_sampler = crate::gpu::passes::mask_weight::make_atlas_sampler(&self.ctx);

            for layer in &effective_layers {
                let eff = crate::cpu::masked::effective_edits_for_layer(&edits, layer);
                let mut eff_uniform = vec![0u8; built.uniform_size];
                write_header(
                    &mut eff_uniform,
                    [sensor_w, sensor_h],
                    [out_w, out_h],
                    [crop.x, crop.y, crop.w, crop.h],
                    [
                        edits.geometry.rotate as u32,
                        edits.geometry.flip_h as u32,
                        edits.geometry.flip_v as u32,
                        orient_packed,
                    ],
                    [lod, shadows_mip_f, 0.0, 0.0],
                    [cos_a, sin_a, bw, bh],
                    [oriented_w as f32, oriented_h as f32, 0.0, 0.0],
                    [
                        crate::tone::tonemap_kind_index(edits.output.tonemap),
                        0,
                        0,
                        0,
                    ],
                );
                let mut active_mask_eff: [u32; 4] = [0; 4];
                for slot in &built.color_ops {
                    let op = &registry.ops()[slot.op_index];
                    if op.is_active(&eff) {
                        let word = (slot.active_bit / 32) as usize;
                        let shift = slot.active_bit % 32;
                        active_mask_eff[word] |= 1u32 << shift;
                    }
                    let mut buf = vec![0.0f32; slot.vec4_count * 4];
                    op.write_gpu_uniform(&eff, &ctx_op, &mut buf);
                    let off = slot.uniform_offset;
                    let bytes = slot.vec4_count * 16;
                    eff_uniform[off..off + bytes].copy_from_slice(bytemuck::cast_slice(&buf));
                }
                write_active_mask(&mut eff_uniform, active_mask_eff);
                let eff_uniform_buf = self.uniform_pool.acquire(device, queue, &eff_uniform, "process-uniform-layer");
                let layer_bind = device.create_bind_group(&BindGroupDescriptor {
                    label: Some("process-bg-layer"),
                    layout: &pass.layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: eff_uniform_buf.as_entire_binding(),
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
                            resource: BindingResource::TextureView(&scratch_tone_view),
                        },
                        BindGroupEntry {
                            binding: 4,
                            resource: BindingResource::TextureView(&scratch_linear_view),
                        },
                        BindGroupEntry {
                            binding: 5,
                            resource: BindingResource::TextureView(shadows_view_ref),
                        },
                    ],
                });
                {
                    let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
                        label: Some("process-layer"),
                        timestamp_writes: None,
                    });
                    cp.set_pipeline(&pass.pipeline);
                    cp.set_bind_group(0, &layer_bind, &[]);
                    let gx = out_w.div_ceil(16);
                    let gy = out_h.div_ceil(16);
                    cp.dispatch_workgroups(gx, gy, 1);
                }
                _retained_uniforms.push(eff_uniform_buf);
                _retained_binds.push(layer_bind);

                let eval = crate::cpu::masked::build_layer_eval(layer, &opts.rasters);
                let (comp_bytes, n_components) =
                    crate::gpu::passes::mask_weight::pack_layer_eval(&eval, &slot_map);
                let lens_warp = crate::ops::lens_distortion::LensWarpParams::from_edits(
                    &edits.lens,
                    display_w,
                    display_h,
                );
                let mw_params = crate::gpu::passes::mask_weight::pack_params(
                    out_w,
                    out_h,
                    n_components,
                    eval.amount,
                    [crop.x, crop.y, crop.w, crop.h],
                    [
                        edits.geometry.rotate as u32,
                        edits.geometry.flip_h as u32,
                        edits.geometry.flip_v as u32,
                        0,
                    ],
                    [cos_a, sin_a, bw, bh],
                    [
                        oriented_w as f32,
                        oriented_h as f32,
                        display_w as f32,
                        display_h as f32,
                    ],
                    [lens_warp.k1, lens_warp.k2, lens_warp.k3, lens_warp.zoom],
                );
                let mw_params_buf = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("mask-weight-uniform"),
                    contents: &mw_params,
                    usage: BufferUsages::UNIFORM,
                });
                let comp_buf_bytes = if comp_bytes.is_empty() {
                    vec![0u8; crate::gpu::passes::mask_weight::COMPONENT_BYTES]
                } else {
                    comp_bytes
                };
                let mw_comp_buf = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("mask-weight-comps"),
                    contents: &comp_buf_bytes,
                    usage: BufferUsages::STORAGE,
                });
                let mw_bind = device.create_bind_group(&BindGroupDescriptor {
                    label: Some("mask-weight-bg"),
                    layout: &self.passes.mask_weight.layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: mw_params_buf.as_entire_binding(),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: mw_comp_buf.as_entire_binding(),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: BindingResource::TextureView(&weight_view),
                        },
                        BindGroupEntry {
                            binding: 3,
                            resource: BindingResource::TextureView(&atlas_view),
                        },
                        BindGroupEntry {
                            binding: 4,
                            resource: BindingResource::Sampler(&atlas_sampler),
                        },
                    ],
                });
                {
                    let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
                        label: Some("mask-weight"),
                        timestamp_writes: None,
                    });
                    cp.set_pipeline(&self.passes.mask_weight.pipeline);
                    cp.set_bind_group(0, &mw_bind, &[]);
                    let gx = out_w.div_ceil(16);
                    let gy = out_h.div_ceil(16);
                    cp.dispatch_workgroups(gx, gy, 1);
                }
                _retained_bufs.push(mw_params_buf);
                _retained_bufs.push(mw_comp_buf);
                _retained_binds.push(mw_bind);

                let (curr_view, dst_view) = if accum_in_alt {
                    (&accum_alt_view, &linear_view2)
                } else {
                    (&linear_view2, &accum_alt_view)
                };
                let bl_params = crate::gpu::passes::mask_blend::pack_params(out_w, out_h);
                let bl_params_buf = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("mask-blend-uniform"),
                    contents: &bl_params,
                    usage: BufferUsages::UNIFORM,
                });
                let bl_bind = device.create_bind_group(&BindGroupDescriptor {
                    label: Some("mask-blend-bg"),
                    layout: &self.passes.mask_blend.layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: bl_params_buf.as_entire_binding(),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(curr_view),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: BindingResource::TextureView(&scratch_linear_view),
                        },
                        BindGroupEntry {
                            binding: 3,
                            resource: BindingResource::TextureView(&weight_view),
                        },
                        BindGroupEntry {
                            binding: 4,
                            resource: BindingResource::TextureView(dst_view),
                        },
                    ],
                });
                {
                    let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
                        label: Some("mask-blend"),
                        timestamp_writes: None,
                    });
                    cp.set_pipeline(&self.passes.mask_blend.pipeline);
                    cp.set_bind_group(0, &bl_bind, &[]);
                    let gx = out_w.div_ceil(16);
                    let gy = out_h.div_ceil(16);
                    cp.dispatch_workgroups(gx, gy, 1);
                }
                _retained_bufs.push(bl_params_buf);
                _retained_binds.push(bl_bind);

                accum_in_alt = !accum_in_alt;
            }
            let _ = scratch_tone_view;
            if accum_in_alt {
                encoder.copy_texture_to_texture(
                    wgpu::ImageCopyTexture {
                        texture: &p.mask_accum_alt,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::ImageCopyTexture {
                        texture: &p.linear_texture,
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
            }
        }

        let sharpen_preview = matches!(
            opts.preview_mode,
            crate::frame::PreviewMode::SharpenMask
                | crate::frame::PreviewMode::SharpenRadius
                | crate::frame::PreviewMode::SharpenDetail
        );
        let final_pass_active = sharpen_active || sharpen_preview || effects_active || has_masks;
        let sharpen_pool_guard = if final_pass_active {
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
            let run_sharpen = sharpen_active || sharpen_preview;
            if run_sharpen {
                self.encode_sharpen(&mut encoder, &edits, p, s, out_w, out_h, &opts.preview_mode);
            }
            if !sharpen_preview {
                self.encode_effects_tone(&mut encoder, &edits, p, s, out_w, out_h, run_sharpen);
            }
        }

        copy_texture_to_buffer(&mut encoder, &p.texture, &p.readback, out_w, out_h);
        let linear_src = match sharpen_pool_guard.as_ref() {
            Some(spool) if !sharpen_preview => {
                &spool.as_ref().expect("sharpen pool populated").post_lin
            }
            _ => &p.linear_texture,
        };
        copy_texture_to_buffer(&mut encoder, linear_src, &p.linear_readback, out_w, out_h);
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
        edits: &Edits,
        out: &OutputTargets,
        sh: &SharpenTargets,
        w: u32,
        h: u32,
        preview: &crate::frame::PreviewMode,
    ) {
        let _span = tracing::debug_span!("gpu.encode_sharpen", w = w, h = h).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let d = &edits.detail;
        let sigma = (d.sharpen_radius as f32).max(0.01);
        let radius = (sigma * 3.0).ceil();
        let sharpen_active = d.sharpen_active();
        let amount = if sharpen_active {
            d.sharpen_amount as f32
        } else {
            0.0
        };
        let detail_weight = 0.5 + 0.5 * (d.sharpen_detail as f32 / 100.0);
        let masking = (d.sharpen_masking as f32 / 100.0).clamp(0.0, 1.0);
        let preview_mode_u: u32 = match preview {
            crate::frame::PreviewMode::None => 0,
            crate::frame::PreviewMode::SharpenMask => 1,
            crate::frame::PreviewMode::SharpenRadius => 2,
            crate::frame::PreviewMode::SharpenDetail => 3,
            crate::frame::PreviewMode::MaskWeight { .. } => 0,
        };
        let use_mask = if (sharpen_active && masking > 0.0) || preview_mode_u == 1 {
            1u32
        } else {
            0u32
        };
        let masking_thresh = masking * 0.5;
        let masking_softness = 0.1f32;

        let linear_view = out
            .linear_texture
            .create_view(&TextureViewDescriptor::default());
        let blur_h_view = sh.blur_h.create_view(&TextureViewDescriptor::default());
        let blur_full_view = sh.blur_full.create_view(&TextureViewDescriptor::default());
        let sharpened_lin_view = sh
            .sharpened_lin
            .create_view(&TextureViewDescriptor::default());
        let out_view = out.texture.create_view(&TextureViewDescriptor::default());

        let pass_h = &self.passes.output_sharpen;

        let gx = w.div_ceil(16);
        let gy = h.div_ceil(16);

        let blur_uniform = |axis: u32| -> super::uniform_pool::PooledUniform {
            let mut bytes = [0u8; 32];
            bytes[0..4].copy_from_slice(&sigma.to_ne_bytes());
            bytes[4..8].copy_from_slice(&radius.to_ne_bytes());
            bytes[8..12].copy_from_slice(&w.to_ne_bytes());
            bytes[12..16].copy_from_slice(&h.to_ne_bytes());
            bytes[16..20].copy_from_slice(&axis.to_ne_bytes());
            self.uniform_pool
                .acquire(device, queue, &bytes, "sharpen-blur-uniform")
        };
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

        let mut sh_bytes = [0u8; 32];
        sh_bytes[0..4].copy_from_slice(&amount.to_ne_bytes());
        sh_bytes[4..8].copy_from_slice(&detail_weight.to_ne_bytes());
        sh_bytes[8..12].copy_from_slice(&masking_thresh.to_ne_bytes());
        sh_bytes[12..16].copy_from_slice(&masking_softness.to_ne_bytes());
        sh_bytes[16..20].copy_from_slice(&w.to_ne_bytes());
        sh_bytes[20..24].copy_from_slice(&h.to_ne_bytes());
        sh_bytes[24..28].copy_from_slice(&use_mask.to_ne_bytes());
        sh_bytes[28..32].copy_from_slice(&preview_mode_u.to_ne_bytes());
        let ub_c = self
            .uniform_pool
            .acquire(device, queue, &sh_bytes, "sharpen-uniform");
        let bg_c = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sharpen-bg"),
            layout: &pass_h.sharpen_layout,
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
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&sharpened_lin_view),
                },
            ],
        });
        let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("sharpen"),
            timestamp_writes: None,
        });
        cp.set_pipeline(&pass_h.sharpen_pipeline);
        cp.set_bind_group(0, &bg_c, &[]);
        cp.dispatch_workgroups(gx, gy, 1);
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_effects_tone(
        &self,
        encoder: &mut CommandEncoder,
        edits: &Edits,
        out: &OutputTargets,
        sh: &SharpenTargets,
        w: u32,
        h: u32,
        sharpen_ran: bool,
    ) {
        let _span = tracing::debug_span!("gpu.encode_effects_tone", w = w, h = h).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let e = &edits.effects;
        let pass = &self.passes.effects_tone;

        let linear_view = out
            .linear_texture
            .create_view(&TextureViewDescriptor::default());
        let sharpened_lin_view = sh
            .sharpened_lin
            .create_view(&TextureViewDescriptor::default());
        let post_lin_view = sh.post_lin.create_view(&TextureViewDescriptor::default());
        let out_view = out.texture.create_view(&TextureViewDescriptor::default());

        let mut bytes = [0u8; 64];
        bytes[0..4].copy_from_slice(&w.to_ne_bytes());
        bytes[4..8].copy_from_slice(&h.to_ne_bytes());
        let vig_amount = (e.vignette_amount / 100.0) as f32;
        let vig_mid = (e.vignette_midpoint / 100.0) as f32;
        let vig_feather = (e.vignette_feather / 100.0) as f32;
        let vig_round = (e.vignette_roundness / 100.0) as f32;
        bytes[16..20].copy_from_slice(&vig_amount.to_ne_bytes());
        bytes[20..24].copy_from_slice(&vig_mid.to_ne_bytes());
        bytes[24..28].copy_from_slice(&vig_feather.to_ne_bytes());
        bytes[28..32].copy_from_slice(&vig_round.to_ne_bytes());
        let gr_amount = (e.grain_amount / 100.0) as f32;
        let gr_size = (e.grain_size / 100.0) as f32;
        let gr_rough = (e.grain_roughness / 100.0) as f32;
        bytes[32..36].copy_from_slice(&gr_amount.to_ne_bytes());
        bytes[36..40].copy_from_slice(&gr_size.to_ne_bytes());
        bytes[40..44].copy_from_slice(&gr_rough.to_ne_bytes());
        let tone_kind = crate::tone::tonemap_kind_index(edits.output.tonemap);
        bytes[48..52].copy_from_slice(&tone_kind.to_ne_bytes());
        let ub = self
            .uniform_pool
            .acquire(device, queue, &bytes, "effects-tone-uniform");
        let src_binding = if sharpen_ran {
            BindingResource::TextureView(&sharpened_lin_view)
        } else {
            BindingResource::TextureView(&linear_view)
        };
        let bg = device.create_bind_group(&BindGroupDescriptor {
            label: Some("effects-tone-bg"),
            layout: &pass.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: ub.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: src_binding,
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&out_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&post_lin_view),
                },
            ],
        });
        let gx = w.div_ceil(16);
        let gy = h.div_ceil(16);
        let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("effects-tone"),
            timestamp_writes: None,
        });
        cp.set_pipeline(&pass.pipeline);
        cp.set_bind_group(0, &bg, &[]);
        cp.dispatch_workgroups(gx, gy, 1);
    }

    pub fn render(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
    ) -> PipelineResult<RenderedImage> {
        self.render_with_cancel(frame, edits, options, None)
    }

    fn run_nr(
        &self,
        src: &Texture,
        dims: (u32, u32),
        edits: &Edits,
    ) -> PipelineResult<Arc<Texture>> {
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

        let mut bytes = vec![0u8; super::passes::nr::NR_UNIFORM_SIZE as usize];
        bytes[0..4].copy_from_slice(&w.to_le_bytes());
        bytes[4..8].copy_from_slice(&h.to_le_bytes());
        bytes[8..12].copy_from_slice(&radius.to_le_bytes());
        bytes[16..20].copy_from_slice(&inv_2ss.to_le_bytes());
        bytes[20..24].copy_from_slice(&inv_2sr_luma.to_le_bytes());
        bytes[24..28].copy_from_slice(&inv_2sr_chroma.to_le_bytes());
        bytes[28..32].copy_from_slice(&alpha_luma.to_le_bytes());
        bytes[32..36].copy_from_slice(&alpha_chroma.to_le_bytes());
        bytes[36..40].copy_from_slice(&contrast.to_le_bytes());
        let uniform_buf = self.uniform_pool.acquire(device, queue, &bytes, "nr-uniform");

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
            return Ok(smoothed);
        }
        Ok(Arc::new(dst))
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

        let mut bytes = vec![0u8; super::passes::nr_smooth::NR_SMOOTH_UNIFORM_SIZE as usize];
        bytes[0..4].copy_from_slice(&w.to_le_bytes());
        bytes[4..8].copy_from_slice(&h.to_le_bytes());
        bytes[16..20].copy_from_slice(&smoothness.to_le_bytes());
        let uniform_buf = self.uniform_pool.acquire(device, queue, &bytes, "nr-smooth-uniform");

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
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
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

    fn atmosphere_for(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        src: &Texture,
        dims: (u32, u32),
    ) -> PipelineResult<[f32; 3]> {
        let key = atmosphere_cache_key(frame, edits, dims);
        if let Some(a) = self.atm_cache.lock().get(&key).copied() {
            tracing::debug!(target: "dehaze", "atm cache hit");
            return Ok(a);
        }
        let _span = tracing::debug_span!("gpu_dehaze_atm", w = dims.0, h = dims.1).entered();
        let atm = self.estimate_atmosphere(src, dims)?;
        self.atm_cache.lock().put(key, atm);
        Ok(atm)
    }

    fn estimate_atmosphere(&self, src: &Texture, dims: (u32, u32)) -> PipelineResult<[f32; 3]> {
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
            wgpu::ImageCopyTexture {
                texture: src,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buf,
                layout: wgpu::ImageDataLayout {
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

        let slice = buf.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = sender.send(r);
        });
        self.ctx.device.poll(wgpu::Maintain::Wait);
        receiver
            .recv()
            .map_err(|e| PipelineError::Render(format!("atm recv: {e}")))?
            .map_err(|e| PipelineError::Render(format!("atm map: {e}")))?;
        let data = slice.get_mapped_range();
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

    fn run_dehaze(
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

        let mut downsample_u = vec![0u8; super::passes::dehaze::DOWNSAMPLE_UNIFORM_SIZE as usize];
        downsample_u[0..4].copy_from_slice(&lw.to_le_bytes());
        downsample_u[4..8].copy_from_slice(&lh.to_le_bytes());
        downsample_u[8..12].copy_from_slice(&scale.to_le_bytes());
        let downsample_buf = self.uniform_pool.acquire(device, queue, &downsample_u, "dehaze-downsample-u");

        let mut norm_u = vec![0u8; 32];
        norm_u[0..4].copy_from_slice(&lw.to_le_bytes());
        norm_u[4..8].copy_from_slice(&lh.to_le_bytes());
        norm_u[16..20].copy_from_slice(&atm[0].to_le_bytes());
        norm_u[20..24].copy_from_slice(&atm[1].to_le_bytes());
        norm_u[24..28].copy_from_slice(&atm[2].to_le_bytes());
        norm_u[28..32].copy_from_slice(&1.0f32.to_le_bytes());
        let norm_buf = self.uniform_pool.acquire(device, queue, &norm_u, "dehaze-norm-u");

        let make_filter_u = |radius: u32, axis: u32, label: &'static str| {
            let mut u = vec![0u8; 16];
            u[0..4].copy_from_slice(&lw.to_le_bytes());
            u[4..8].copy_from_slice(&lh.to_le_bytes());
            u[8..12].copy_from_slice(&radius.to_le_bytes());
            u[12..16].copy_from_slice(&axis.to_le_bytes());
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some(label),
                contents: &u,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };
        let min_h_buf = make_filter_u(r_patch, 0, "dehaze-min-h-u");
        let min_v_buf = make_filter_u(r_patch, 1, "dehaze-min-v-u");
        let box_h_buf = make_filter_u(r_gf, 0, "dehaze-box-h-u");
        let box_v_buf = make_filter_u(r_gf, 1, "dehaze-box-v-u");

        let mut size_u = vec![0u8; 16];
        size_u[0..4].copy_from_slice(&lw.to_le_bytes());
        size_u[4..8].copy_from_slice(&lh.to_le_bytes());
        let pack_buf = self.uniform_pool.acquire(device, queue, &size_u, "dehaze-pack-u");
        let ab_uni = self.uniform_pool.acquire(device, queue, &size_u, "dehaze-ab-u");

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
        let apply_buf = self.uniform_pool.acquire(device, queue, &apply_u, "dehaze-apply-u");

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

    fn run_presence(
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
        let uniform_buf = self.uniform_pool.acquire(device, queue, &uniform_bytes, "presence-uniform");

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

    fn run_wb_prepare(
        &self,
        cached: &CachedFrame,
        frame: &RawFrame,
        edits: &Edits,
    ) -> PipelineResult<Arc<Texture>> {
        let _span = tracing::debug_span!("gpu.run_wb_prepare", w = cached.width, h = cached.height)
            .entered();
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
            render: RenderContext {
                wb_coeffs: frame.wb_coeffs,
                cam_to_srgb,
                is_raw: frame.is_raw,
                preview_mode: crate::frame::PreviewMode::None,
            },
            scratch: OpScratch { shadows_blur: None },
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

        let uniform_buf = self.uniform_pool.acquire(device, queue, &uniform_bytes, "wb-prepare-uniform");

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

        Ok(Arc::new(wb_base))
    }

    fn run_sensor(
        &self,
        src: &Arc<CachedFrame>,
        edits: &Edits,
    ) -> PipelineResult<Arc<CachedFrame>> {
        use super::passes::sensor::SensorParams;

        let _span = tracing::debug_span!("gpu.run_sensor", w = src.width, h = src.height).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = src.width;
        let h = src.height;
        let params = SensorParams::from_edits(&edits.lens, w, h);
        let uniform_buf = self.uniform_pool.acquire(device, queue, bytemuck::bytes_of(&params), "sensor-uniform");
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
        let cached = if edits.lens.any_active() {
            let corrected = self.run_sensor(&cached, &edits.clamped())?;
            crate::cancel::check(cancel)?;
            corrected
        } else {
            cached
        };
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
                    None,
                )?;
                crate::cancel::check(cancel)?;
                Ok(out)
            }
            RenderPlan::Presence => {
                let wb_base = self.run_wb_prepare(&cached, frame, &edits_c)?;
                crate::cancel::check(cancel)?;
                let nr_out = if edits_c.detail.luma_nr_active() || edits_c.detail.color_nr_active()
                {
                    let t = self.run_nr(&wb_base, dims, &edits_c)?;
                    crate::cancel::check(cancel)?;
                    Some(t)
                } else {
                    None
                };
                let presence_src: &Texture = nr_out.as_deref().unwrap_or(wb_base.as_ref());
                let dehaze_out: Option<Arc<Texture>> = if edits_c.basic.dehaze != 0.0 {
                    let atm = self.atmosphere_for(frame, &edits_c, presence_src, dims)?;
                    let _span =
                        tracing::debug_span!("gpu_dehaze", w = dims.0, h = dims.1).entered();
                    let t = self.run_dehaze(presence_src, dims, &edits_c, atm)?;
                    crate::cancel::check(cancel)?;
                    Some(t)
                } else {
                    None
                };
                let post_dehaze_src: &Texture = dehaze_out.as_deref().unwrap_or(presence_src);
                let presence_active = edits_c.basic.texture != 0.0 || edits_c.basic.clarity != 0.0;
                let processed_src: Arc<Texture> = if presence_active {
                    self.run_presence(post_dehaze_src, dims, &edits_c)?
                } else if let Some(t) = dehaze_out {
                    t
                } else {
                    nr_out.unwrap_or(wb_base)
                };
                crate::cancel::check(cancel)?;
                let shadows_pyramid = if edits_c.tone.shadows != 0.0 {
                    Some(self.build_luma_pyramid(&processed_src, dims)?)
                } else {
                    None
                };
                let shadows_view = shadows_pyramid
                    .as_ref()
                    .map(|t| t.create_view(&TextureViewDescriptor::default()));
                let out = self.process(
                    &self.passes.process_post_wb,
                    &processed_src,
                    dims,
                    frame,
                    edits,
                    options,
                    shadows_view.as_ref(),
                )?;
                crate::cancel::check(cancel)?;
                drop(shadows_view);
                drop(shadows_pyramid);
                Ok(out)
            }
        }
    }

    fn build_luma_pyramid(&self, src: &Texture, dims: (u32, u32)) -> PipelineResult<Arc<Texture>> {
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

fn make_dummy_luma(ctx: &GpuContext) -> Texture {
    let tex = ctx.device.create_texture(&TextureDescriptor {
        label: Some("shadows-blur-dummy"),
        size: Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: ctx.linear_format,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let zero = [0u8; 8];
    ctx.queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &zero,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(8),
            rows_per_image: Some(1),
        },
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    tex
}

fn atmosphere_cache_key(frame: &RawFrame, edits: &Edits, dims: (u32, u32)) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut e = edits.clone();
    e.basic.dehaze = 0.0;
    let json = serde_json::to_vec(&e).unwrap_or_default();
    let mut h = std::collections::hash_map::DefaultHasher::new();
    GpuRenderer::frame_key(frame).hash(&mut h);
    dims.0.hash(&mut h);
    dims.1.hash(&mut h);
    json.hash(&mut h);
    h.finish()
}
