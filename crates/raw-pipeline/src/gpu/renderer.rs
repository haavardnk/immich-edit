use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

use parking_lot::Mutex;
use wgpu::{
    Extent3d, Texture, TextureDescriptor, TextureDimension, TextureUsages, TextureViewDescriptor,
};

use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions, RenderedImage};
use crate::{PipelineError, PipelineResult};

use super::context::GpuContext;
use super::passes::GpuPasses;
use super::resources::{OutputTargets, SharpenTargets};
use super::texture_pool::TexturePool;
use super::timer::RenderTimings;
use super::uniform_pool::UniformPool;
use crate::timing;

mod cache_keys;
mod dcp;
mod detail;
mod display;
mod effects;
mod geometry;
mod lut;
mod masks;
mod meta;
mod output;
mod pools;
mod resample;
mod retouch;
mod stage_cache;
mod uniform;
mod upload;

use cache_keys::StageKeys;
use display::{DisplayFrame, DisplayInput};
use geometry::compute_out_dims;
pub use pools::GpuPoolStats;
use stage_cache::{Stage, StageCache};

use super::budget::GpuBudget;
use super::display_depth::DisplayDepth;

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
    block: usize,
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
            || crate::ops::capture_sharpen::frame_sigma(frame, edits, 1).is_some()
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
    superpixel_cache: Mutex<lru::LruCache<u64, Arc<CachedFrame>>>,
    atm_cache: Mutex<lru::LruCache<u64, [f32; 3]>>,
    stages: StageCache,
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
    dummy_luma: Texture,
}

const ATM_CACHE_ITEMS: usize = 16;
const LUT_TEX_CACHE_ITEMS: usize = 4;
const HUESAT_TEX_CACHE_ITEMS: usize = 4;

const ATLAS_CACHE_ITEMS: usize = 32;
const ATLAS_POOL_ITEMS: usize = 2;
const TEXTURE_POOL_CAP_PER_KEY: usize = 4;
const UNIFORM_POOL_CAP_PER_SIZE: usize = 8;
const DEFAULT_TEXTURE_CACHE_MAX_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub struct GpuRendererOptions {
    pub texture_cache_max_bytes: u64,
    pub timestamps: bool,
}

impl Default for GpuRendererOptions {
    fn default() -> Self {
        Self {
            texture_cache_max_bytes: DEFAULT_TEXTURE_CACHE_MAX_BYTES,
            timestamps: false,
        }
    }
}

impl GpuRenderer {
    pub fn new() -> PipelineResult<Self> {
        Self::with_options(GpuRendererOptions::default())
    }

    pub fn with_options(options: GpuRendererOptions) -> PipelineResult<Self> {
        let ctx = GpuContext::with_timestamps(options.timestamps)?;
        let passes = Arc::new(GpuPasses::new(&ctx));
        let budget = GpuBudget::new(options.texture_cache_max_bytes);
        let texture_pool = TexturePool::new(TEXTURE_POOL_CAP_PER_KEY, budget.clone());
        let dummy_luma = make_dummy_luma(&ctx);
        Ok(Self {
            ctx,
            passes,
            cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(CACHE_ITEMS).expect("nonzero"),
            )),
            superpixel_cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(CACHE_ITEMS).expect("nonzero"),
            )),
            atm_cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(ATM_CACHE_ITEMS).expect("nonzero"),
            )),
            stages: StageCache::new(budget, texture_pool.clone()),
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
            texture_pool,
            uniform_pool: UniformPool::new(UNIFORM_POOL_CAP_PER_SIZE),
            output_pool: Mutex::new(Vec::new()),
            sharpen_pool: Mutex::new(Vec::new()),
            dummy_luma,
        })
    }

    pub fn adapter_label(&self) -> String {
        self.ctx.adapter_label()
    }

    pub fn is_software_adapter(&self) -> bool {
        self.ctx.is_software()
    }

    pub fn gpu_timestamps(&self) -> bool {
        self.ctx.timestamps
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
    fn spatial_base(
        &self,
        cached: &CachedFrame,
        dims: (u32, u32),
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        setup: &crate::dcp_pipeline::DcpSetup,
        t: &RenderTimings,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<((u32, u32), Arc<Texture>)> {
        let keys = StageKeys::new(frame, edits, dims, setup.cam_to_srgb);
        let wb_base = t.stage(timing::WB_PREPARE, || {
            self.run_wb_prepare(cached, frame, edits, setup, keys.wb)
        })?;
        crate::cancel::check(cancel)?;
        let wb_base = if edits.retouch.iter().any(|s| s.is_effective()) {
            let tex = t.stage(timing::RETOUCH, || {
                self.run_retouch(wb_base, dims, frame, edits)
            })?;
            crate::cancel::check(cancel)?;
            tex
        } else {
            wb_base
        };
        let full_src: Arc<Texture> =
            if edits.detail.luma_nr_active() || edits.detail.color_nr_active() {
                let tex = t.stage(timing::NOISE_REDUCTION, || {
                    self.run_nr(&wb_base, dims, edits, keys.nr)
                })?;
                crate::cancel::check(cancel)?;
                tex
            } else {
                wb_base
            };
        let sigma = crate::ops::capture_sharpen::frame_sigma(frame, edits, cached.block)
            .filter(|_| dims.0 >= 8 && dims.1 >= 8);
        let full_src: Arc<Texture> = match sigma {
            Some(sigma) => {
                let tex = t.stage(timing::CAPTURE_SHARPEN, || {
                    self.run_capture_sharpen(&full_src, dims, sigma, keys.capture(sigma))
                })?;
                crate::cancel::check(cancel)?;
                tex
            }
            None => full_src,
        };
        let preview_dims = crate::geom::preview_ratio(
            frame.meta.orientation,
            edits,
            (frame.meta.width as u32, frame.meta.height as u32),
            options.max_edge,
            options.quality,
        )
        .and_then(|ratio| crate::geom::resample_target(dims, ratio / cached.block as f32));
        let (spatial_dims, spatial_src) = match preview_dims {
            Some(preview_dims) => {
                let downsampled = t.stage(timing::PREVIEW_RESAMPLE, || {
                    self.resample_lanczos(&full_src, dims, preview_dims, "preview-spatial-src")
                })?;
                crate::cancel::check(cancel)?;
                (preview_dims, downsampled)
            }
            None => (dims, full_src),
        };
        let base = if edits.basic.dehaze != 0.0 {
            let key = keys.spatial(sigma, spatial_dims);
            let tex = t.stage(timing::DEHAZE, || {
                let atm = self.atmosphere_for(key, spatial_src.as_ref(), spatial_dims, cancel)?;
                let _span =
                    tracing::debug_span!("gpu_dehaze", w = spatial_dims.0, h = spatial_dims.1)
                        .entered();
                self.run_dehaze(spatial_src.as_ref(), spatial_dims, edits, atm)
            })?;
            crate::cancel::check(cancel)?;
            tex
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
        t: &RenderTimings,
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
            if let Some(tex) = cache.get(&key) {
                out.insert(layer.id.clone(), tex.clone());
                continue;
            }
            let layer_base = if wb == global_wb {
                base.clone()
            } else if let Some(tex) = base_cache.get(&wb) {
                tex.clone()
            } else {
                let (_, tex) =
                    self.spatial_base(cached, dims, frame, &eff, options, setup, t, cancel)?;
                base_cache.insert(wb, tex.clone());
                tex
            };
            let tex = if amts.texture == 0.0 && amts.clarity == 0.0 {
                layer_base
            } else {
                let tex = t.stage(timing::PRESENCE, || {
                    self.run_presence(&layer_base, spatial_dims, &eff)
                })?;
                crate::cancel::check(cancel)?;
                tex
            };
            cache.insert(key, tex.clone());
            out.insert(layer.id.clone(), tex);
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
        let timings = RenderTimings::new(&self.ctx);
        let display = self.render_display(frame, edits, options, timings, cancel)?;
        let image = self.readback_image(display, cancel)?;
        output::finish_image(image, options, frame.meta.is_raw, cancel)
    }

    fn render_display<'a>(
        &'a self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        timings: RenderTimings<'a>,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<DisplayFrame<'a>> {
        let t = &timings;
        let mut composed = edits.clamped();
        composed.geometry.crop = crate::geom::compose_roi(composed.geometry.crop, options.roi);
        let edits = &composed;
        let plan = RenderPlan::select(edits, frame);
        let full_dims = (frame.meta.width as u32, frame.meta.height as u32);
        let preview_ratio = crate::geom::preview_ratio(
            frame.meta.orientation,
            edits,
            full_dims,
            options.max_edge,
            options.quality,
        );
        let block = crate::geom::superpixel_block(frame, options, preview_ratio);
        let cached = t.stage(timing::DEMOSAIC, || self.get_or_demosaic(frame, block))?;
        crate::cancel::check(cancel)?;
        let cached = if edits.lens.any_active() {
            let corrected = t.stage(timing::LENS, || self.run_sensor(&cached, &edits.clamped()))?;
            crate::cancel::check(cancel)?;
            corrected
        } else {
            cached
        };
        let dims = (cached.width, cached.height);
        let edits_c = edits.clamped();
        let depth = display_depth(options);
        let out_dims = compute_out_dims(frame, &edits_c, full_dims, options.max_edge);
        match plan {
            RenderPlan::Fast => {
                let pass = match depth {
                    DisplayDepth::Eight => &self.passes.process_fast,
                    DisplayDepth::Sixteen => &self.passes.depth16(&self.ctx).process_fast,
                };
                let input = DisplayInput {
                    pass,
                    src: cached.texture.as_ref(),
                    src_dims: dims,
                    out_dims,
                    frame,
                    edits,
                    opts: options,
                    shadows: None,
                    layer_srcs: &HashMap::new(),
                };
                self.encode_display(input, timings, cancel)
            }
            RenderPlan::Presence => {
                let setup = crate::dcp_pipeline::resolve(frame, &edits_c, options.dcp.as_deref());
                let (spatial_dims, base_src) =
                    self.spatial_base(&cached, dims, frame, &edits_c, options, &setup, t, cancel)?;
                let presence_active = edits_c.basic.texture != 0.0 || edits_c.basic.clarity != 0.0;
                let processed_src: Arc<Texture> = if presence_active {
                    t.stage(timing::PRESENCE, || {
                        self.run_presence(&base_src, spatial_dims, &edits_c)
                    })?
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
                    t,
                    cancel,
                )?;
                let shadows_pyramid = if edits_c.tone.shadows != 0.0 {
                    Some(t.stage(timing::SHADOWS, || {
                        self.build_luma_pyramid(&processed_src, spatial_dims)
                    })?)
                } else {
                    None
                };
                let shadows_view = shadows_pyramid
                    .as_ref()
                    .map(|t| t.create_view(&TextureViewDescriptor::default()));
                let pass = match depth {
                    DisplayDepth::Eight => &self.passes.process_post_wb,
                    DisplayDepth::Sixteen => &self.passes.depth16(&self.ctx).process_post_wb,
                };
                let input = DisplayInput {
                    pass,
                    src: &processed_src,
                    src_dims: spatial_dims,
                    out_dims,
                    frame,
                    edits,
                    opts: options,
                    shadows: shadows_view.as_ref(),
                    layer_srcs: &layer_srcs,
                };
                self.encode_display(input, timings, cancel)
            }
        }
    }
}

fn make_dummy_luma(ctx: &GpuContext) -> Texture {
    let bytes_per_texel = ctx.linear_format.block_copy_size(None).unwrap_or(8);
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
    ctx.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &vec![0u8; bytes_per_texel as usize],
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_texel),
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
