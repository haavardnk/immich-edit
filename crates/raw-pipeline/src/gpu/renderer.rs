use std::num::NonZeroUsize;
use std::sync::Arc;

use parking_lot::Mutex;
use wgpu::{Extent3d, Texture, TextureDescriptor, TextureDimension, TextureUsages};

#[cfg(feature = "native")]
use crate::PipelineError;
use crate::PipelineResult;
use crate::edits::Edits;
use crate::frame::RenderOptions;
#[cfg(feature = "native")]
use crate::frame::{RawFrame, RenderedImage};

use super::budget::GpuBudget;
use super::context::GpuContext;
use super::display_depth::DisplayDepth;
use super::passes::GpuPasses;
use super::resources::{OutputTargets, SharpenTargets};
#[cfg(feature = "native")]
use super::source::RenderSource;
use super::texture_pool::TexturePool;
#[cfg(feature = "native")]
use super::timer::RenderTimings;
use super::uniform_pool::UniformPool;

#[cfg(feature = "native")]
mod cache_keys;
mod dcp;
mod detail;
mod display;
mod effects;
mod geometry;
mod lut;
mod mask_overlay;
mod masks;
mod meta;
mod mipgen;
#[cfg(feature = "native")]
mod output;
mod pools;
mod resample;
#[cfg(feature = "native")]
mod retouch;
#[cfg(feature = "native")]
mod sensor_stage;
#[cfg(feature = "native")]
mod stage_cache;
mod uniform;
#[cfg(feature = "native")]
mod upload;
#[cfg(feature = "web")]
mod web;

pub use display::DisplayFrame;
pub use pools::GpuPoolStats;
#[cfg(feature = "native")]
use sensor_stage::{CachedFrame, SensorCaches};
#[cfg(feature = "native")]
use stage_cache::Stage;
#[cfg(feature = "web")]
pub use web::DisplayMeta;

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

fn compose_edits(edits: &Edits, options: &RenderOptions) -> Edits {
    let mut composed = edits.clamped();
    composed.geometry.crop = crate::geom::compose_roi(composed.geometry.crop, options.roi);
    composed
}

fn presence_display(edits: &Edits) -> bool {
    let b = &edits.basic;
    let masked_presence = edits
        .masks
        .iter()
        .filter(|l| l.is_effective())
        .any(|l| l.edits.texture.is_some() || l.edits.clarity.is_some());
    b.texture != 0.0
        || b.clarity != 0.0
        || b.dehaze != 0.0
        || edits.tone.shadows != 0.0
        || masked_presence
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPlan {
    Fast,
    Presence,
}

impl RenderPlan {
    #[cfg(feature = "native")]
    pub fn select(edits: &Edits, frame: &RawFrame) -> Self {
        let d = &edits.detail;
        if presence_display(edits)
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

pub struct GpuRenderer {
    ctx: Arc<GpuContext>,
    passes: Arc<GpuPasses>,
    #[cfg(feature = "native")]
    sensor: SensorCaches,
    lut_tex_cache: Mutex<lru::LruCache<u64, Arc<Texture>>>,
    huesat_tex_cache: Mutex<lru::LruCache<u64, Arc<Texture>>>,
    atlas_cache: Mutex<lru::LruCache<String, Arc<Vec<u8>>>>,
    atlas_pool: Mutex<Vec<masks::MaskAtlas>>,
    atlas_allocs: std::sync::atomic::AtomicU64,
    atlas_uploads: std::sync::atomic::AtomicU64,
    texture_pool: Arc<TexturePool>,
    uniform_pool: Arc<UniformPool>,
    output_pool: Mutex<Vec<OutputTargets>>,
    sharpen_pool: Mutex<Vec<SharpenTargets>>,
    dummy_luma: Texture,
}

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
    #[cfg(feature = "native")]
    pub fn new() -> PipelineResult<Self> {
        Self::with_options(GpuRendererOptions::default())
    }

    #[cfg(feature = "native")]
    pub fn with_options(options: GpuRendererOptions) -> PipelineResult<Self> {
        pollster::block_on(Self::new_async(options))
    }

    pub async fn new_async(options: GpuRendererOptions) -> PipelineResult<Self> {
        let ctx = GpuContext::new_async(options.timestamps).await?;
        let passes = Arc::new(GpuPasses::new(&ctx));
        let budget = GpuBudget::new(options.texture_cache_max_bytes);
        let texture_pool = TexturePool::new(TEXTURE_POOL_CAP_PER_KEY, budget.clone());
        let dummy_luma = make_dummy_luma(&ctx);
        Ok(Self {
            ctx,
            passes,
            #[cfg(feature = "native")]
            sensor: SensorCaches::new(budget, texture_pool.clone()),
            lut_tex_cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(LUT_TEX_CACHE_ITEMS).expect("nonzero"),
            )),
            huesat_tex_cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(HUESAT_TEX_CACHE_ITEMS).expect("nonzero"),
            )),
            atlas_cache: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(ATLAS_CACHE_ITEMS).expect("nonzero"),
            )),
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

    pub fn mask_atlas_allocations(&self) -> u64 {
        self.atlas_allocs.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn mask_atlas_uploads(&self) -> u64 {
        self.atlas_uploads
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    #[cfg(feature = "native")]
    pub fn render<'s>(
        &self,
        source: impl Into<RenderSource<'s>>,
        edits: &Edits,
        options: &RenderOptions,
    ) -> PipelineResult<RenderedImage> {
        self.render_with_cancel(source, edits, options, None)
    }

    #[cfg(feature = "native")]
    pub fn render_with_cancel<'s>(
        &self,
        source: impl Into<RenderSource<'s>>,
        edits: &Edits,
        options: &RenderOptions,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<RenderedImage> {
        if self.ctx.is_lost() {
            return Err(PipelineError::DeviceLost);
        }
        crate::cancel::check(cancel)?;
        let source = source.into();
        let timings = RenderTimings::new(&self.ctx);
        let edits = compose_edits(edits, options);
        let display = match source {
            RenderSource::Raw(frame) => {
                let linear = self.sensor_stage(frame, &edits, options, &timings, cancel)?;
                self.display_chain(&linear, &edits, options, timings, cancel)?
            }
            RenderSource::Linear(linear) => {
                self.display_chain(linear, &edits, options, timings, cancel)?
            }
        };
        let image = self.readback_image(display, cancel)?;
        output::finish_image(image, options, source.meta().is_raw, cancel)
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
