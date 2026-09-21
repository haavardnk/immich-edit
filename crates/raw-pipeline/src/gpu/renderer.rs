use std::num::NonZeroUsize;
use std::sync::Arc;

use parking_lot::Mutex;
use wgpu::{
    CommandEncoderDescriptor, Extent3d, Texture, TextureDescriptor, TextureDimension,
    TextureUsages, TextureViewDescriptor,
};

use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions, RenderedImage};
use crate::gpu::dispatch::{bind_group, dispatch_2d, samp, tex};
use crate::ops::{GpuRoute, OpContext, OpScratch, RenderContext};
use crate::{PipelineError, PipelineResult};

use super::context::GpuContext;
use super::passes::GpuPasses;
use super::resources::{OutputTargets, SharpenTargets};
use super::texture_pool::TexturePool;
use super::uniform_pool::UniformPool;
use crate::presence::{presence_mips, presence_radii};

mod cache_keys;
mod dcp;
mod detail;
mod effects;
mod geometry;
mod lut;
mod masks;
mod output;
mod pools;
mod resample;
mod retouch;
mod uniform;
mod upload;

use cache_keys::{capture_cache_key, spatial_cache_key};
use geometry::{compute_out_dims, crop_px, process_geom};
pub use pools::GpuPoolStats;

use super::display_depth::DisplayDepth;
use super::texture_pool::TextureKey;

const CACHE_ITEMS: usize = 2;

fn display_depth(opts: &RenderOptions) -> DisplayDepth {
    if opts.gamut_warn
        || opts.clip_warn
        || matches!(
            opts.preview_mode,
            crate::frame::PreviewMode::MaskWeight { .. }
        )
    {
        return DisplayDepth::Eight;
    }
    DisplayDepth::for_output(&opts.output)
}

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

fn wb_delta(edits: &Edits) -> (u64, u64) {
    (edits.basic.wb_temp.to_bits(), edits.basic.wb_tint.to_bits())
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
    atlas_pool: Mutex<Vec<masks::MaskAtlas>>,
    atlas_allocs: std::sync::atomic::AtomicU64,
    atlas_uploads: std::sync::atomic::AtomicU64,
    atm_estimates: std::sync::atomic::AtomicU64,
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
const ATLAS_POOL_ITEMS: usize = 2;
const TEXTURE_POOL_CAP_PER_KEY: usize = 4;
const UNIFORM_POOL_CAP_PER_SIZE: usize = 8;
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
            atm_estimates: std::sync::atomic::AtomicU64::new(0),
            atlas_pool: Mutex::new(Vec::new()),
            atlas_allocs: std::sync::atomic::AtomicU64::new(0),
            atlas_uploads: std::sync::atomic::AtomicU64::new(0),
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

    pub fn is_lost(&self) -> bool {
        self.ctx.is_lost()
    }

    pub fn atmosphere_estimates(&self) -> u64 {
        self.atm_estimates
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn mask_atlas_allocations(&self) -> u64 {
        self.atlas_allocs.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn mask_atlas_uploads(&self) -> u64 {
        self.atlas_uploads
            .load(std::sync::atomic::Ordering::Relaxed)
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
            if op.gpu_route() != GpuRoute::Fused {
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

        let downscaled = match crate::geom::resample_target(src_dims, ratio) {
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
        let geom = process_geom(frame, &edits, work_dims);

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
        let uniform_bytes = uniform::build_process_uniform(
            built,
            registry,
            &edits,
            &ctx_op,
            &uniform::process_header(
                &edits,
                &geom,
                (sensor_w, sensor_h),
                out_dims,
                shadows_mip_f,
                true,
            ),
        );

        let uniform_buf =
            self.uniform_pool
                .acquire(device, queue, &uniform_bytes, "process-uniform");

        let src_view = src_texture.create_view(&TextureViewDescriptor::default());

        let pool = pools::acquire_target(&self.output_pool, &self.ctx, out_w, out_h);
        let p = &pool[0];
        let depth = display_depth(opts);
        let display_target = (depth == DisplayDepth::Sixteen).then(|| {
            self.texture_pool.acquire(
                &self.ctx.device,
                TextureKey::new(
                    depth.format(),
                    out_w,
                    out_h,
                    1,
                    TextureUsages::STORAGE_BINDING
                        | TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST,
                ),
                "display-16",
            )
        });
        let display_tex: &Texture = display_target
            .as_ref()
            .map(|t| t.texture())
            .unwrap_or(&p.texture);
        let out_view = display_tex.create_view(&TextureViewDescriptor::default());
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

        let bind = bind_group(
            device,
            "process-bg",
            &pass.layout,
            &[
                uniform_buf.as_entire_binding(),
                tex(&src_view),
                samp(&self.passes.linear_sampler),
                tex(&out_view),
                tex(&linear_view),
                tex(shadows_view_ref),
            ],
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("process-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "process-pass",
            &pass.pipeline,
            &bind,
            out_w.div_ceil(16),
            out_h.div_ceil(16),
        );

        let mask_stage = self.encode_mask_stage(
            &mut encoder,
            masks::MaskStage {
                pass,
                edits: &edits,
                opts,
                geom: &geom,
                ctx_op: &ctx_op,
                target: p,
                src_view: &src_view,
                linear_view: &linear_view,
                shadows_view: shadows_view_ref,
                layer_srcs,
                sensor_dims: (sensor_w, sensor_h),
                out_dims,
                shadows_mip: shadows_mip_f,
                masked_sharpen,
            },
        );
        let masks::MaskStageOutput {
            mut retained,
            preview_atlas,
            layer_atlas,
            has_masks,
            preview_active,
        } = mask_stage;

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
        let sharpen_pool_guard = final_pass_active
            .then(|| pools::acquire_target(&self.sharpen_pool, &self.ctx, out_w, out_h));
        let _huesat_scratch = self.run_dcp_base_table(
            &mut encoder,
            ctx_op.render.dcp.as_deref(),
            &p.linear_texture,
            out_w,
            out_h,
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
            self.encode_effects_tone(
                &mut encoder,
                &edits,
                p,
                s,
                display_tex,
                depth,
                out_w,
                out_h,
                run_sharpen,
                opts.output_color_space,
                warn_flags,
                opts.roi,
            );
        }

        let _dcp_finish_scratch = sharpen_pool_guard.as_ref().and_then(|spool| {
            self.run_dcp_finish(
                &mut encoder,
                ctx_op.render.dcp.as_deref(),
                &spool[0].post_lin,
                display_tex,
                depth,
                out_w,
                out_h,
                warn_flags | ((p3_active as u32) << 2),
            )
        });

        let lut_target =
            self.maybe_encode_lut(&mut encoder, &edits, opts, display_tex, depth, out_w, out_h);
        let display_src = lut_target.as_deref().unwrap_or(display_tex);
        let overlay = preview_active;
        if overlay {
            self.encode_mask_overlay(&mut encoder, p, display_src, out_dims, &mut retained);
        }
        let display_src = if overlay {
            &p.mask_scratch_tone
        } else {
            display_src
        };
        let display_readback = (depth == DisplayDepth::Sixteen).then(|| {
            crate::gpu::readback::make_readback_buffer_wide(device, "readback-16", out_w, out_h)
        });
        let linear_src = opts.histogram.then(|| match sharpen_pool_guard.as_ref() {
            Some(spool) => &spool[0].post_lin,
            _ => &p.linear_texture,
        });
        let (rgba, linear_rgb) = self.readback_image(
            encoder,
            p,
            display_src,
            display_readback.as_ref(),
            linear_src,
            out_dims,
            cancel,
        )?;
        self.release_mask_atlas(preview_atlas);
        self.release_mask_atlas(layer_atlas);
        drop(lut_target);
        drop(display_target);
        drop(pool);

        output::finish_image(rgba, linear_rgb, out_dims, geom.source, opts, frame.is_raw)
    }

    #[allow(clippy::too_many_arguments)]
    fn spatial_base(
        &self,
        cached: &CachedFrame,
        dims: (u32, u32),
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        setup: &crate::dcp_pipeline::DcpSetup,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<((u32, u32), Arc<Texture>)> {
        let wb_base = self.run_wb_prepare(cached, frame, edits, setup)?;
        crate::cancel::check(cancel)?;
        let wb_base = if edits.retouch.iter().any(|s| s.is_effective()) {
            let t = self.run_retouch(wb_base, dims, frame, edits)?;
            crate::cancel::check(cancel)?;
            t
        } else {
            wb_base
        };
        let full_src: Arc<Texture> =
            if edits.detail.luma_nr_active() || edits.detail.color_nr_active() {
                let t = self.run_nr(&wb_base, dims, edits, frame, setup.cam_to_srgb)?;
                crate::cancel::check(cancel)?;
                t
            } else {
                wb_base
            };
        let sigma = crate::ops::capture_sharpen::frame_sigma(frame, edits)
            .filter(|_| dims.0 >= 8 && dims.1 >= 8);
        let full_src: Arc<Texture> = match sigma {
            Some(sigma) => {
                let key = capture_cache_key(frame, edits, dims, setup.cam_to_srgb, sigma);
                let t = self.run_capture_sharpen(&full_src, dims, sigma, key)?;
                crate::cancel::check(cancel)?;
                t
            }
            None => full_src,
        };
        let preview_dims = crate::geom::preview_ratio(
            frame.orientation,
            edits,
            dims,
            options.max_edge,
            options.quality,
        )
        .and_then(|ratio| crate::geom::resample_target(dims, ratio));
        let (spatial_dims, spatial_src) = match preview_dims {
            Some(preview_dims) => {
                let downsampled =
                    self.resample_lanczos(&full_src, dims, preview_dims, "preview-spatial-src")?;
                crate::cancel::check(cancel)?;
                (preview_dims, downsampled)
            }
            None => (dims, full_src),
        };
        let base = if edits.basic.dehaze != 0.0 {
            let key = spatial_cache_key(frame, edits, dims, setup.cam_to_srgb, sigma, spatial_dims);
            let atm = self.atmosphere_for(key, spatial_src.as_ref(), spatial_dims, cancel)?;
            let _span = tracing::debug_span!("gpu_dehaze", w = spatial_dims.0, h = spatial_dims.1)
                .entered();
            let t = self.run_dehaze(spatial_src.as_ref(), spatial_dims, edits, atm)?;
            crate::cancel::check(cancel)?;
            t
        } else {
            spatial_src
        };
        Ok((spatial_dims, base))
    }

    #[allow(clippy::too_many_arguments)]
    fn layer_presence_sources(
        &self,
        cached: &CachedFrame,
        dims: (u32, u32),
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        setup: &crate::dcp_pipeline::DcpSetup,
        base: &Arc<Texture>,
        spatial_dims: (u32, u32),
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<std::collections::HashMap<String, Arc<Texture>>> {
        let mut out = std::collections::HashMap::new();
        let global_amts = crate::presence::presence_amounts(edits);
        let global_wb = wb_delta(edits);
        let mut base_cache: std::collections::HashMap<(u64, u64), Arc<Texture>> =
            std::collections::HashMap::new();
        let mut cache: std::collections::HashMap<(u64, u64, u32, u32), Arc<Texture>> =
            std::collections::HashMap::new();
        for layer in edits.masks.iter().filter(|l| l.is_effective()) {
            let eff = crate::cpu::masked::effective_edits_for_layer(edits, layer);
            let amts = crate::presence::presence_amounts(&eff);
            let wb = wb_delta(&eff);
            if wb == global_wb
                && amts.texture == global_amts.texture
                && amts.clarity == global_amts.clarity
            {
                continue;
            }
            let key = (wb.0, wb.1, amts.texture.to_bits(), amts.clarity.to_bits());
            if let Some(t) = cache.get(&key) {
                out.insert(layer.id.clone(), t.clone());
                continue;
            }
            let layer_base = if wb == global_wb {
                base.clone()
            } else if let Some(t) = base_cache.get(&wb) {
                t.clone()
            } else {
                let (_, t) =
                    self.spatial_base(cached, dims, frame, &eff, options, setup, cancel)?;
                base_cache.insert(wb, t.clone());
                t
            };
            let t = if amts.texture == 0.0 && amts.clarity == 0.0 {
                layer_base
            } else {
                let t = self.run_presence(&layer_base, spatial_dims, &eff)?;
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
        let depth = display_depth(options);
        match plan {
            RenderPlan::Fast => {
                let pass = match depth {
                    DisplayDepth::Eight => &self.passes.process_fast,
                    DisplayDepth::Sixteen => &self.passes.depth16(&self.ctx).process_fast,
                };
                let out = self.process(
                    pass,
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
                let (out_w, out_h) = compute_out_dims(frame, &edits_c, dims, options.max_edge);
                let (spatial_dims, base_src) =
                    self.spatial_base(&cached, dims, frame, &edits_c, options, &setup, cancel)?;
                let presence_active = edits_c.basic.texture != 0.0 || edits_c.basic.clarity != 0.0;
                let processed_src: Arc<Texture> = if presence_active {
                    self.run_presence(&base_src, spatial_dims, &edits_c)?
                } else {
                    base_src.clone()
                };
                crate::cancel::check(cancel)?;
                let layer_srcs = self.layer_presence_sources(
                    &cached,
                    dims,
                    frame,
                    &edits_c,
                    options,
                    &setup,
                    &base_src,
                    spatial_dims,
                    cancel,
                )?;
                let shadows_pyramid = if edits_c.tone.shadows != 0.0 {
                    Some(self.build_luma_pyramid(&processed_src, spatial_dims)?)
                } else {
                    None
                };
                let shadows_view = shadows_pyramid
                    .as_ref()
                    .map(|t| t.create_view(&TextureViewDescriptor::default()));
                let out = self.process(
                    match depth {
                        DisplayDepth::Eight => &self.passes.process_post_wb,
                        DisplayDepth::Sixteen => &self.passes.depth16(&self.ctx).process_post_wb,
                    },
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
