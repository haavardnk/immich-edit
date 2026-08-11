use std::num::NonZeroUsize;
use std::sync::Arc;

use parking_lot::Mutex;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, BufferUsages, CommandEncoderDescriptor,
    ComputePassDescriptor, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureUsages,
    TextureViewDescriptor,
};

use crate::edits::Edits;
use crate::encode::encode_from_rgba8;
use crate::frame::{RawFrame, RenderOptions, RenderedImage};
use crate::histogram::Histogram;
use crate::ops::{GpuOpKind, OpContext, OpScratch, RenderContext};
use crate::{PipelineError, PipelineResult};

use super::context::GpuContext;
use super::passes::GpuPasses;
use super::readback::{copy_texture_to_buffer, read_rgba8, read_rgba16f_as_rgb};
use super::resources::{OutputTargets, SharpenTargets};
use super::texture_pool::TexturePool;
use super::uniform_pool::UniformPool;
use super::uniforms::{write_active_mask, write_header};
use crate::presence::{presence_mips, presence_radii};

mod dcp;
mod detail;
mod effects;
mod lut;
mod resample;
mod retouch;
mod upload;

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
    pub fn select(edits: &Edits, frame: &RawFrame) -> Self {
        let b = &edits.basic;
        let d = &edits.detail;
        let masked_presence = edits
            .masks
            .iter()
            .filter(|l| l.is_effective())
            .any(|l| l.edits.texture.is_some() || l.edits.clarity.is_some());
        if b.texture != 0.0
            || b.clarity != 0.0
            || b.dehaze != 0.0
            || edits.tone.shadows != 0.0
            || masked_presence
            || d.luma_nr_active()
            || d.color_nr_active()
            || crate::ops::capture_sharpen::frame_sigma(frame, edits).is_some()
            || edits.retouch.iter().any(|s| s.is_effective())
        {
            Self::Presence
        } else {
            Self::Fast
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GpuPoolStats {
    pub texture_pool: u64,
    pub uniform_pool: u64,
    pub output_targets: u64,
    pub sharpen_targets: u64,
    pub wb_cache: u64,
    pub nr_cache: u64,
    pub capture_cache: u64,
    pub atlas_cache: u64,
}

fn texture_bytes(tex: &Texture) -> u64 {
    let bpp = tex.format().block_copy_size(None).unwrap_or(0) as u64;
    let w = tex.width() as u64;
    let h = tex.height() as u64;
    let mips = tex.mip_level_count();
    let mut total: u64 = 0;
    for level in 0..mips {
        let lw = (w >> level).max(1);
        let lh = (h >> level).max(1);
        total += lw * lh * bpp;
    }
    total
}

fn output_targets_bytes(o: &OutputTargets) -> u64 {
    texture_bytes(&o.texture)
        + o.readback.size()
        + texture_bytes(&o.linear_texture)
        + o.linear_readback.size()
        + texture_bytes(&o.mask_accum_alt)
        + texture_bytes(&o.mask_scratch_linear)
        + texture_bytes(&o.mask_scratch_tone)
        + texture_bytes(&o.mask_weight)
}

fn sharpen_targets_bytes(s: &SharpenTargets) -> u64 {
    texture_bytes(&s.blur_h)
        + texture_bytes(&s.blur_full)
        + texture_bytes(&s.sharpened_lin)
        + texture_bytes(&s.post_lin)
}

pub struct GpuRenderer {
    ctx: Arc<GpuContext>,
    passes: Arc<GpuPasses>,
    cache: Mutex<lru::LruCache<u64, Arc<CachedFrame>>>,
    atm_cache: Mutex<lru::LruCache<u64, [f32; 3]>>,
    wb_cache: Mutex<lru::LruCache<u64, Arc<Texture>>>,
    nr_cache: Mutex<lru::LruCache<u64, Arc<Texture>>>,
    capture_cache: Mutex<lru::LruCache<u64, Arc<Texture>>>,
    lut_tex_cache: Mutex<lru::LruCache<u64, Arc<Texture>>>,
    huesat_tex_cache: Mutex<lru::LruCache<u64, Arc<Texture>>>,
    atlas_cache: Mutex<lru::LruCache<String, Arc<Vec<u8>>>>,
    texture_pool: Arc<TexturePool>,
    uniform_pool: Arc<UniformPool>,
    output_pool: Mutex<Vec<OutputTargets>>,
    sharpen_pool: Mutex<Vec<SharpenTargets>>,
}

const ATM_CACHE_ITEMS: usize = 16;
const WB_CACHE_ITEMS: usize = 2;
const NR_CACHE_ITEMS: usize = 2;
const CAPTURE_CACHE_ITEMS: usize = 2;
const LUT_TEX_CACHE_ITEMS: usize = 4;
const HUESAT_TEX_CACHE_ITEMS: usize = 4;

const ATLAS_CACHE_ITEMS: usize = 32;
const TEXTURE_POOL_CAP_PER_KEY: usize = 4;
const UNIFORM_POOL_CAP_PER_SIZE: usize = 8;
const TARGET_POOL_CAP: usize = 2;
const DEFAULT_TEXTURE_POOL_MAX_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub struct GpuRendererOptions {
    pub texture_pool_max_bytes: u64,
}

impl Default for GpuRendererOptions {
    fn default() -> Self {
        Self {
            texture_pool_max_bytes: DEFAULT_TEXTURE_POOL_MAX_BYTES,
        }
    }
}

impl GpuRenderer {
    pub fn new() -> PipelineResult<Self> {
        Self::with_options(GpuRendererOptions::default())
    }

    pub fn with_options(options: GpuRendererOptions) -> PipelineResult<Self> {
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
            wb_cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(WB_CACHE_ITEMS).expect("nonzero"),
            )),
            nr_cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(NR_CACHE_ITEMS).expect("nonzero"),
            )),
            capture_cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(CAPTURE_CACHE_ITEMS).expect("nonzero"),
            )),
            lut_tex_cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(LUT_TEX_CACHE_ITEMS).expect("nonzero"),
            )),
            huesat_tex_cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(HUESAT_TEX_CACHE_ITEMS).expect("nonzero"),
            )),
            atlas_cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(ATLAS_CACHE_ITEMS).expect("nonzero"),
            )),
            texture_pool: TexturePool::new(
                TEXTURE_POOL_CAP_PER_KEY,
                options.texture_pool_max_bytes,
            ),
            uniform_pool: UniformPool::new(UNIFORM_POOL_CAP_PER_SIZE),
            output_pool: Mutex::new(Vec::new()),
            sharpen_pool: Mutex::new(Vec::new()),
        })
    }

    pub fn adapter_label(&self) -> String {
        self.ctx.adapter_label()
    }

    pub fn pool_stats(&self) -> GpuPoolStats {
        let output_bytes = self
            .output_pool
            .lock()
            .iter()
            .map(output_targets_bytes)
            .sum();
        let sharpen_bytes = self
            .sharpen_pool
            .lock()
            .iter()
            .map(sharpen_targets_bytes)
            .sum();
        let wb_cache_bytes = self
            .wb_cache
            .lock()
            .iter()
            .map(|(_, t)| texture_bytes(t))
            .sum();
        let nr_cache_bytes = self
            .nr_cache
            .lock()
            .iter()
            .map(|(_, t)| texture_bytes(t))
            .sum();
        let capture_cache_bytes = self
            .capture_cache
            .lock()
            .iter()
            .map(|(_, t)| texture_bytes(t))
            .sum();
        let atlas_cache_bytes = self
            .atlas_cache
            .lock()
            .iter()
            .map(|(_, v)| v.len() as u64)
            .sum();
        GpuPoolStats {
            texture_pool: self.texture_pool.bytes(),
            uniform_pool: self.uniform_pool.bytes(),
            output_targets: output_bytes,
            sharpen_targets: sharpen_bytes,
            wb_cache: wb_cache_bytes,
            nr_cache: nr_cache_bytes,
            capture_cache: capture_cache_bytes,
            atlas_cache: atlas_cache_bytes,
        }
    }

    pub fn is_lost(&self) -> bool {
        self.ctx.is_lost()
    }

    fn upload_mask_atlas(
        &self,
        slot_map: &std::collections::HashMap<String, u32>,
        rasters: &crate::mask_raster::RasterMap,
    ) -> wgpu::Texture {
        let atlas = self.ctx.device.create_texture(&wgpu::TextureDescriptor {
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
        for (raster_id, slot) in slot_map {
            let Some(raster) = rasters.get(raster_id) else {
                continue;
            };
            let bytes = {
                let mut cache = self.atlas_cache.lock();
                if let Some(b) = cache.get(raster_id).cloned() {
                    b
                } else {
                    let b = Arc::new(crate::gpu::passes::mask_weight::resample_raster_to_atlas(
                        raster,
                    ));
                    cache.put(raster_id.clone(), b.clone());
                    b
                }
            };
            self.ctx.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &atlas,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: *slot,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                bytes.as_slice(),
                wgpu::TexelCopyBufferLayout {
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
        atlas
    }

    #[allow(clippy::too_many_arguments)]
    fn process(
        &self,
        pass: &super::passes::process::ProcessFastPass,
        src_texture: &Texture,
        src_dims: (u32, u32),
        out_dims: (u32, u32),
        frame: &RawFrame,
        edits: &Edits,
        opts: &RenderOptions,
        shadows_blur: Option<&wgpu::TextureView>,
        layer_srcs: &std::collections::HashMap<String, Arc<Texture>>,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<RenderedImage> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;

        let mut edits = edits.clamped();
        edits.detail.sharpen_amount = Some(edits.detail.sharpen_amount_for(frame.is_raw));
        let edits = edits;
        let sharpen_active = edits.detail.sharpen_active();
        let masked_sharpen = edits.masked_sharpen_active();
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
            if op.id() == "retouch" {
                continue;
            }
            if op.id() == crate::ops::dcp_profile::DCP_PROFILE_OP_ID {
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

        let (out_w, out_h) = out_dims;
        let (crop_w_px, crop_h_px) = crop_px(frame, &edits, src_dims);
        let ratio = (crop_w_px as f32 / out_w as f32).max(crop_h_px as f32 / out_h as f32);

        let downscaled = match resample::resample_target(src_dims, ratio) {
            Some(dims) => Some((
                self.resample_lanczos(src_texture, src_dims, dims, "process-downscale")?,
                dims,
            )),
            None => None,
        };
        let downscaled_layers: std::collections::HashMap<String, Arc<Texture>> = match &downscaled {
            Some((_, dims)) => layer_srcs
                .iter()
                .map(|(id, tex)| {
                    self.resample_lanczos(tex, src_dims, *dims, "layer-downscale")
                        .map(|t| (id.clone(), t))
                })
                .collect::<PipelineResult<_>>()?,
            None => std::collections::HashMap::new(),
        };
        let (src_texture, work_dims, layer_srcs) = match &downscaled {
            Some((tex, dims)) => (tex.as_ref(), *dims, &downscaled_layers),
            None => (src_texture, src_dims, layer_srcs),
        };
        crate::cancel::check(cancel)?;

        let (sensor_w, sensor_h) = work_dims;
        let (display_w, display_h) = if frame.orientation.0 {
            (sensor_h, sensor_w)
        } else {
            (sensor_w, sensor_h)
        };

        let (oriented_w, oriented_h) = match edits.geometry.rotate {
            90 | 270 => (display_h, display_w),
            _ => (display_w, display_h),
        };

        let (full_display_w, full_display_h) = if frame.orientation.0 {
            (frame.height as u32, frame.width as u32)
        } else {
            (frame.width as u32, frame.height as u32)
        };
        let (source_w, source_h) = match edits.geometry.rotate {
            90 | 270 => (full_display_h, full_display_w),
            _ => (full_display_w, full_display_h),
        };

        let crop = edits
            .geometry
            .crop
            .unwrap_or(crate::edits::CropRect::full());
        let angle = edits.geometry.rotate_angle;
        let bbox = crate::geom::rotated_bbox(oriented_w as f32, oriented_h as f32, angle);
        let bw = bbox.w;
        let bh = bbox.h;

        let a_rad = crate::geom::deg_to_rad(angle);
        let cos_a = a_rad.cos();
        let sin_a = a_rad.sin();
        let persp_rows = crate::perspective::mat3_rows(&edits.geometry.perspective_inverse());

        let (ot, oh_h, oh_v) = frame.orientation;
        let orient_packed = (oh_h as u32) | ((oh_v as u32) << 1) | ((ot as u32) << 2);
        let geom_warps = !crop.is_full()
            || angle.abs() > 1e-4
            || edits.geometry.perspective_inverse() != crate::perspective::IDENTITY;

        let setup = crate::dcp_pipeline::resolve(frame, &edits, opts.dcp.as_deref());
        let ctx_op = OpContext {
            render: RenderContext {
                wb_coeffs: frame.wb_coeffs,
                cam_to_srgb: setup.cam_to_srgb,
                is_raw: frame.is_raw,
                capture_sigma: frame.capture_sigma,
                preview_mode: opts.preview_mode.clone(),
                roi: opts.roi,
                dcp: setup.resolved,
            },
            scratch: OpScratch::default(),
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
            [0.0, shadows_mip_f, 0.0, 0.0],
            [cos_a, sin_a, bw, bh],
            [
                oriented_w as f32,
                oriented_h as f32,
                if geom_warps { 1.0 } else { 0.0 },
                0.0,
            ],
            [0, 0, 0, 0],
            persp_rows,
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

        let uniform_buf =
            self.uniform_pool
                .acquire(device, queue, &uniform_bytes, "process-uniform");

        let src_view = src_texture.create_view(&TextureViewDescriptor::default());

        let mut pool = self.output_pool.lock();
        if let Some(i) = pool.iter().position(|p| p.fits(out_w, out_h)) {
            if i != 0 {
                let t = pool.remove(i);
                pool.insert(0, t);
            }
        } else {
            if pool.len() >= TARGET_POOL_CAP {
                pool.pop();
            }
            pool.insert(0, OutputTargets::allocate(&self.ctx, out_w, out_h));
        }
        let p = &pool[0];
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
                    resource: BindingResource::Sampler(&self.passes.linear_sampler),
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

        let preview_layer = match &opts.preview_mode {
            crate::frame::PreviewMode::MaskWeight { layer_id } => {
                edits.masks.iter().find(|l| &l.id == layer_id)
            }
            _ => None,
        };
        let effective_layers: Vec<&crate::edits::MaskLayer> = if preview_layer.is_some() {
            Vec::new()
        } else {
            edits.masks.iter().filter(|l| l.is_effective()).collect()
        };
        let has_masks = !effective_layers.is_empty();
        let mut accum_in_alt = false;
        let mut _retained_bufs: Vec<wgpu::Buffer> = Vec::new();
        let mut _retained_uniforms: Vec<super::uniform_pool::PooledUniform> = Vec::new();
        let mut _retained_binds: Vec<wgpu::BindGroup> = Vec::new();
        let mut _preview_atlas: Option<wgpu::Texture> = None;
        if let Some(layer) = preview_layer {
            let mut slot_map: std::collections::HashMap<String, u32> =
                std::collections::HashMap::new();
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
            let atlas = self.upload_mask_atlas(&slot_map, &opts.rasters);
            let atlas_view = atlas.create_view(&TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });
            let atlas_sampler = &self.passes.atlas_sampler;
            let weight_view = p.mask_weight.create_view(&TextureViewDescriptor::default());
            let eval = crate::cpu::masked::build_layer_eval(layer, &opts.rasters);
            let (comp_bytes, n_components, poly_bytes) =
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
                    eval.invert as u32,
                ],
                [cos_a, sin_a, bw, bh],
                [
                    oriented_w as f32,
                    oriented_h as f32,
                    display_w as f32,
                    display_h as f32,
                ],
                [lens_warp.k1, lens_warp.k2, lens_warp.k3, lens_warp.zoom],
                persp_rows,
            );
            let mw_params_buf = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("mask-preview-uniform"),
                contents: &mw_params,
                usage: BufferUsages::UNIFORM,
            });
            let comp_buf_bytes = if comp_bytes.is_empty() {
                vec![0u8; crate::gpu::passes::mask_weight::COMPONENT_BYTES]
            } else {
                comp_bytes
            };
            let mw_comp_buf = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("mask-preview-comps"),
                contents: &comp_buf_bytes,
                usage: BufferUsages::STORAGE,
            });
            let poly_buf_bytes = if poly_bytes.is_empty() {
                vec![0u8; 8]
            } else {
                poly_bytes
            };
            let mw_poly_buf = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("mask-preview-poly"),
                contents: &poly_buf_bytes,
                usage: BufferUsages::STORAGE,
            });
            let mw_bind = device.create_bind_group(&BindGroupDescriptor {
                label: Some("mask-preview-bg"),
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
                        resource: BindingResource::Sampler(atlas_sampler),
                    },
                    BindGroupEntry {
                        binding: 5,
                        resource: BindingResource::TextureView(&out_view),
                    },
                    BindGroupEntry {
                        binding: 6,
                        resource: mw_poly_buf.as_entire_binding(),
                    },
                ],
            });
            {
                let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("mask-preview-weight"),
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
            _retained_bufs.push(mw_poly_buf);
            _retained_binds.push(mw_bind);
            _preview_atlas = Some(atlas);
        }
        if has_masks {
            let scratch_linear_view = p
                .mask_scratch_linear
                .create_view(&TextureViewDescriptor::default());
            let scratch_tone_view = p
                .mask_scratch_tone
                .create_view(&TextureViewDescriptor::default());
            let weight_view = p.mask_weight.create_view(&TextureViewDescriptor::default());
            let sharpen_accum_view = p
                .mask_sharpen
                .create_view(&TextureViewDescriptor::default());
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
            let atlas = self.upload_mask_atlas(&slot_map, &opts.rasters);
            let atlas_view = atlas.create_view(&wgpu::TextureViewDescriptor {
                label: Some("mask-raster-atlas-view"),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });
            let atlas_sampler = &self.passes.atlas_sampler;

            for (layer_index, layer) in effective_layers.iter().enumerate() {
                let eff = crate::cpu::masked::effective_edits_for_layer(&edits, layer);
                let layer_src_view = layer_srcs
                    .get(&layer.id)
                    .map(|t| t.create_view(&TextureViewDescriptor::default()));
                let layer_src_view_ref = layer_src_view.as_ref().unwrap_or(&src_view);
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
                    [0.0, shadows_mip_f, 0.0, 0.0],
                    [cos_a, sin_a, bw, bh],
                    [oriented_w as f32, oriented_h as f32, 0.0, 0.0],
                    [0, 0, 0, 0],
                    persp_rows,
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
                let eff_uniform_buf =
                    self.uniform_pool
                        .acquire(device, queue, &eff_uniform, "process-uniform-layer");
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
                            resource: BindingResource::TextureView(layer_src_view_ref),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: BindingResource::Sampler(&self.passes.linear_sampler),
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
                let (comp_bytes, n_components, poly_bytes) =
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
                        eval.invert as u32,
                    ],
                    [cos_a, sin_a, bw, bh],
                    [
                        oriented_w as f32,
                        oriented_h as f32,
                        display_w as f32,
                        display_h as f32,
                    ],
                    [lens_warp.k1, lens_warp.k2, lens_warp.k3, lens_warp.zoom],
                    persp_rows,
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
                let poly_buf_bytes = if poly_bytes.is_empty() {
                    vec![0u8; 8]
                } else {
                    poly_bytes
                };
                let mw_poly_buf = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("mask-weight-poly"),
                    contents: &poly_buf_bytes,
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
                            resource: BindingResource::Sampler(atlas_sampler),
                        },
                        BindGroupEntry {
                            binding: 5,
                            resource: BindingResource::TextureView(&out_view),
                        },
                        BindGroupEntry {
                            binding: 6,
                            resource: mw_poly_buf.as_entire_binding(),
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
                _retained_bufs.push(mw_poly_buf);
                _retained_binds.push(mw_bind);

                let (curr_view, dst_view) = if accum_in_alt {
                    (&accum_alt_view, &linear_view2)
                } else {
                    (&linear_view2, &accum_alt_view)
                };
                let sharpen_flags = if !masked_sharpen {
                    0u32
                } else if layer_index == 0 {
                    1u32
                } else {
                    2u32
                };
                let bl_params = crate::gpu::passes::mask_blend::pack_params(
                    out_w,
                    out_h,
                    layer.edits.sharpen.unwrap_or(0.0) as f32,
                    sharpen_flags,
                );
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
                        BindGroupEntry {
                            binding: 5,
                            resource: BindingResource::TextureView(&sharpen_accum_view),
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
                    wgpu::TexelCopyTextureInfo {
                        texture: &p.mask_accum_alt,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
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
        let dcp_active = ctx_op.render.dcp.is_some();
        let p3_active = matches!(
            opts.output_color_space,
            crate::frame::OutputColorSpace::DisplayP3
        );
        let final_pass_active = sharpen_active
            || sharpen_preview
            || effects_active
            || has_masks
            || dcp_active
            || p3_active
            || opts.gamut_warn
            || opts.clip_warn;
        let warn_flags = opts.gamut_warn as u32 | ((opts.clip_warn as u32) << 1);
        let sharpen_pool_guard = if final_pass_active {
            let mut spool = self.sharpen_pool.lock();
            if let Some(i) = spool.iter().position(|s| s.fits(out_w, out_h)) {
                if i != 0 {
                    let t = spool.remove(i);
                    spool.insert(0, t);
                }
            } else {
                if spool.len() >= TARGET_POOL_CAP {
                    spool.pop();
                }
                spool.insert(0, SharpenTargets::allocate(&self.ctx, out_w, out_h));
            }
            Some(spool)
        } else {
            None
        };
        let _huesat_scratch = self.run_dcp_base_table(
            &mut encoder,
            ctx_op.render.dcp.as_deref(),
            &p.linear_texture,
            out_w,
            out_h,
            sharpen_preview,
        );
        if let Some(spool) = sharpen_pool_guard.as_ref() {
            let s = &spool[0];
            let run_sharpen = sharpen_active || masked_sharpen || sharpen_preview;
            if run_sharpen {
                self.encode_sharpen(
                    &mut encoder,
                    &edits,
                    p,
                    s,
                    out_w,
                    out_h,
                    &opts.preview_mode,
                    masked_sharpen,
                );
            }
            if !sharpen_preview {
                self.encode_effects_tone(
                    &mut encoder,
                    &edits,
                    p,
                    s,
                    out_w,
                    out_h,
                    run_sharpen,
                    dcp_active,
                    opts.output_color_space,
                    warn_flags,
                    opts.roi,
                );
            }
        }

        let _dcp_finish_scratch = sharpen_pool_guard.as_ref().and_then(|spool| {
            self.run_dcp_finish(
                &mut encoder,
                ctx_op.render.dcp.as_deref(),
                &spool[0].post_lin,
                &p.texture,
                out_w,
                out_h,
                sharpen_preview,
                warn_flags | ((p3_active as u32) << 2),
            )
        });

        let lut_target = if sharpen_preview {
            None
        } else {
            self.maybe_encode_lut(&mut encoder, &edits, opts, &p.texture, out_w, out_h)
        };
        let display_src = lut_target.as_deref().unwrap_or(&p.texture);
        let overlay_bind = preview_layer.map(|_| {
            let params = crate::gpu::passes::mask_overlay::pack_params(
                out_w,
                out_h,
                crate::gpu::passes::mask_overlay::OVERLAY_ALPHA,
            );
            let params_buf = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("mask-overlay-uniform"),
                contents: &params,
                usage: BufferUsages::UNIFORM,
            });
            let src_view = display_src.create_view(&TextureViewDescriptor::default());
            let weight_view = p.mask_weight.create_view(&TextureViewDescriptor::default());
            let dst_view = p
                .mask_scratch_tone
                .create_view(&TextureViewDescriptor::default());
            let bind = device.create_bind_group(&BindGroupDescriptor {
                label: Some("mask-overlay-bg"),
                layout: &self.passes.mask_overlay.layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: params_buf.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&src_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&weight_view),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::TextureView(&dst_view),
                    },
                ],
            });
            (params_buf, bind)
        });
        if let Some((_, bind)) = overlay_bind.as_ref() {
            let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("mask-overlay"),
                timestamp_writes: None,
            });
            cp.set_pipeline(&self.passes.mask_overlay.pipeline);
            cp.set_bind_group(0, bind, &[]);
            let gx = out_w.div_ceil(16);
            let gy = out_h.div_ceil(16);
            cp.dispatch_workgroups(gx, gy, 1);
        }
        let display_src = if overlay_bind.is_some() {
            &p.mask_scratch_tone
        } else {
            display_src
        };
        copy_texture_to_buffer(&mut encoder, display_src, &p.readback, out_w, out_h);
        let linear_src = match sharpen_pool_guard.as_ref() {
            Some(spool) if !sharpen_preview => &spool[0].post_lin,
            _ => &p.linear_texture,
        };
        copy_texture_to_buffer(&mut encoder, linear_src, &p.linear_readback, out_w, out_h);
        queue.submit(Some(encoder.finish()));

        let mut rgba = read_rgba8(&self.ctx, &p.readback, out_w, out_h, cancel)?;
        let linear_rgb = read_rgba16f_as_rgb(&self.ctx, &p.linear_readback, out_w, out_h, cancel)?;
        drop(lut_target);
        drop(pool);

        if opts.gamut_warn || opts.clip_warn {
            crate::warn::paint_rgba8(&mut rgba, opts.gamut_warn, opts.clip_warn);
        }

        let ((histogram, linear_histogram), bytes) = rayon::join(
            || {
                let _span = tracing::debug_span!("gpu.histogram", w = out_w, h = out_h).entered();
                rayon::join(
                    || {
                        let _s =
                            tracing::debug_span!("gpu.histogram.display", w = out_w, h = out_h)
                                .entered();
                        Histogram::from_rgba8(&rgba)
                    },
                    || {
                        let _s = tracing::debug_span!("gpu.histogram.linear", w = out_w, h = out_h)
                            .entered();
                        Histogram::from_rgb(&linear_rgb, out_w as usize, out_h as usize)
                    },
                )
            },
            || encode_from_rgba8(&rgba, out_w, out_h, &opts.output, opts.output_color_space),
        );
        let bytes = bytes?;

        Ok(RenderedImage {
            bytes,
            histogram,
            linear_histogram: Some(linear_histogram),
            width: out_w,
            height: out_h,
            source_w,
            source_h,
            renderer: "gpu".into(),
            is_raw: frame.is_raw,
        })
    }

    fn layer_presence_sources(
        &self,
        src: &Arc<Texture>,
        dims: (u32, u32),
        edits: &Edits,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<std::collections::HashMap<String, Arc<Texture>>> {
        let mut out = std::collections::HashMap::new();
        let global_amts = crate::presence::presence_amounts(edits);
        let mut cache: std::collections::HashMap<(u32, u32), Arc<Texture>> =
            std::collections::HashMap::new();
        for layer in edits.masks.iter().filter(|l| l.is_effective()) {
            let eff = crate::cpu::masked::effective_edits_for_layer(edits, layer);
            let amts = crate::presence::presence_amounts(&eff);
            if amts.texture == global_amts.texture && amts.clarity == global_amts.clarity {
                continue;
            }
            let key = (amts.texture.to_bits(), amts.clarity.to_bits());
            if let Some(t) = cache.get(&key) {
                out.insert(layer.id.clone(), t.clone());
                continue;
            }
            let t = if amts.texture == 0.0 && amts.clarity == 0.0 {
                src.clone()
            } else {
                let t = self.run_presence(src, dims, &eff)?;
                crate::cancel::check(cancel)?;
                t
            };
            cache.insert(key, t.clone());
            out.insert(layer.id.clone(), t);
        }
        Ok(out)
    }

    pub fn render(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
    ) -> PipelineResult<RenderedImage> {
        self.render_with_cancel(frame, edits, options, None)
    }

    pub fn render_with_cancel(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<RenderedImage> {
        if self.ctx.is_lost() {
            return Err(PipelineError::DeviceLost);
        }
        crate::cancel::check(cancel)?;
        let mut composed = edits.clamped();
        composed.geometry.crop = crate::geom::compose_roi(composed.geometry.crop, options.roi);
        let edits = &composed;
        let plan = RenderPlan::select(edits, frame);
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
                    compute_out_dims(frame, &edits_c, dims, options.max_edge),
                    frame,
                    edits,
                    options,
                    None,
                    &std::collections::HashMap::new(),
                    cancel,
                )?;
                crate::cancel::check(cancel)?;
                Ok(out)
            }
            RenderPlan::Presence => {
                let setup = crate::dcp_pipeline::resolve(frame, &edits_c, options.dcp.as_deref());
                let wb_base = self.run_wb_prepare(&cached, frame, &edits_c, &setup)?;
                crate::cancel::check(cancel)?;
                let (out_w, out_h) = compute_out_dims(frame, &edits_c, dims, options.max_edge);
                let (crop_w_px, crop_h_px) = crop_px(frame, &edits_c, dims);
                let out_max = out_w.max(out_h);
                let ratio = (crop_w_px as f32 / out_w as f32).max(crop_h_px as f32 / out_h as f32);
                let wb_base = if edits_c.retouch.iter().any(|s| s.is_effective()) {
                    let t = self.run_retouch(wb_base, dims, frame, &edits_c)?;
                    crate::cancel::check(cancel)?;
                    t
                } else {
                    wb_base
                };
                let full_src: Arc<Texture> =
                    if edits_c.detail.luma_nr_active() || edits_c.detail.color_nr_active() {
                        let t = self.run_nr(&wb_base, dims, &edits_c, frame, setup.cam_to_srgb)?;
                        crate::cancel::check(cancel)?;
                        t
                    } else {
                        wb_base
                    };
                let full_src: Arc<Texture> =
                    match crate::ops::capture_sharpen::frame_sigma(frame, &edits_c)
                        .filter(|_| dims.0 >= 8 && dims.1 >= 8)
                    {
                        Some(sigma) => {
                            let key =
                                capture_cache_key(frame, &edits_c, dims, setup.cam_to_srgb, sigma);
                            let t = self.run_capture_sharpen(&full_src, dims, sigma, key)?;
                            crate::cancel::check(cancel)?;
                            t
                        }
                        None => full_src,
                    };
                let preview_active = !options.quality && out_max >= 256 && ratio >= 2.0;
                let preview_dims = preview_active
                    .then(|| resample::resample_target(dims, ratio))
                    .flatten();
                let (spatial_dims, spatial_src) = match preview_dims {
                    Some(preview_dims) => {
                        let downsampled = self.resample_lanczos(
                            &full_src,
                            dims,
                            preview_dims,
                            "preview-spatial-src",
                        )?;
                        crate::cancel::check(cancel)?;
                        (preview_dims, downsampled)
                    }
                    None => (dims, full_src),
                };
                let dehaze_out: Option<Arc<Texture>> = if edits_c.basic.dehaze != 0.0 {
                    let atm = self.atmosphere_for(
                        frame,
                        &edits_c,
                        spatial_src.as_ref(),
                        spatial_dims,
                        cancel,
                    )?;
                    let _span =
                        tracing::debug_span!("gpu_dehaze", w = spatial_dims.0, h = spatial_dims.1)
                            .entered();
                    let t = self.run_dehaze(spatial_src.as_ref(), spatial_dims, &edits_c, atm)?;
                    crate::cancel::check(cancel)?;
                    Some(t)
                } else {
                    None
                };
                let base_src: Arc<Texture> = dehaze_out.unwrap_or(spatial_src);
                let presence_active = edits_c.basic.texture != 0.0 || edits_c.basic.clarity != 0.0;
                let processed_src: Arc<Texture> = if presence_active {
                    self.run_presence(&base_src, spatial_dims, &edits_c)?
                } else {
                    base_src.clone()
                };
                crate::cancel::check(cancel)?;
                let layer_srcs =
                    self.layer_presence_sources(&base_src, spatial_dims, &edits_c, cancel)?;
                let shadows_pyramid = if edits_c.tone.shadows != 0.0 {
                    Some(self.build_luma_pyramid(&processed_src, spatial_dims)?)
                } else {
                    None
                };
                let shadows_view = shadows_pyramid
                    .as_ref()
                    .map(|t| t.create_view(&TextureViewDescriptor::default()));
                let out = self.process(
                    &self.passes.process_post_wb,
                    &processed_src,
                    spatial_dims,
                    (out_w, out_h),
                    frame,
                    edits,
                    options,
                    shadows_view.as_ref(),
                    &layer_srcs,
                    cancel,
                )?;
                crate::cancel::check(cancel)?;
                drop(shadows_view);
                drop(shadows_pyramid);
                Ok(out)
            }
        }
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
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &zero,
        wgpu::TexelCopyBufferLayout {
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

fn wb_cache_key(
    frame: &RawFrame,
    edits: &Edits,
    dims: (u32, u32),
    cam_to_srgb: [[f32; 3]; 3],
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    GpuRenderer::frame_key(frame).hash(&mut h);
    dims.0.hash(&mut h);
    dims.1.hash(&mut h);
    edits.basic.wb_temp.to_bits().hash(&mut h);
    edits.basic.wb_tint.to_bits().hash(&mut h);
    for row in cam_to_srgb {
        for v in row {
            v.to_bits().hash(&mut h);
        }
    }
    let lens_json = serde_json::to_vec(&edits.lens).unwrap_or_default();
    lens_json.hash(&mut h);
    h.finish()
}

fn nr_cache_key(
    frame: &RawFrame,
    edits: &Edits,
    dims: (u32, u32),
    cam_to_srgb: [[f32; 3]; 3],
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    wb_cache_key(frame, edits, dims, cam_to_srgb).hash(&mut h);
    let retouch_json = serde_json::to_vec(&edits.retouch).unwrap_or_default();
    retouch_json.hash(&mut h);
    let d = &edits.detail;
    d.luma_nr_amount.to_bits().hash(&mut h);
    d.luma_nr_detail.to_bits().hash(&mut h);
    d.luma_nr_contrast.to_bits().hash(&mut h);
    d.color_nr_amount.to_bits().hash(&mut h);
    d.color_nr_detail.to_bits().hash(&mut h);
    d.color_nr_smoothness.to_bits().hash(&mut h);
    h.finish()
}

fn capture_cache_key(
    frame: &RawFrame,
    edits: &Edits,
    dims: (u32, u32),
    cam_to_srgb: [[f32; 3]; 3],
    sigma: f32,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    nr_cache_key(frame, edits, dims, cam_to_srgb).hash(&mut h);
    sigma.to_bits().hash(&mut h);
    h.finish()
}

fn compute_out_dims(
    frame: &RawFrame,
    edits: &Edits,
    src_dims: (u32, u32),
    max_edge: u32,
) -> (u32, u32) {
    crate::geom::display_out_dims(frame.orientation, edits, src_dims, max_edge)
}

fn crop_px(frame: &RawFrame, edits: &Edits, src_dims: (u32, u32)) -> (u32, u32) {
    crate::geom::display_crop_px(frame.orientation, edits, src_dims)
}
