use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::Mutex;
use wgpu::Texture;

use super::cache_keys::StageKeys;
use super::stage_cache::StageCache;
use super::{GpuRenderer, RenderPlan};
use crate::PipelineResult;
use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions};
use crate::gpu::budget::GpuBudget;
use crate::gpu::source::LinearSource;
use crate::gpu::texture_pool::TexturePool;
use crate::gpu::timer::RenderTimings;
use crate::source::{LinearKind, RenderedSource};
use crate::timing;

const FRAME_CACHE_ITEMS: usize = 2;
const ATMOSPHERE_CACHE_ITEMS: usize = 16;

#[derive(Clone, Copy)]
pub(super) struct SourcePlan {
    render: RenderPlan,
    atmosphere: bool,
}

impl SourcePlan {
    const PORTABLE: Self = Self {
        render: RenderPlan::Presence,
        atmosphere: true,
    };

    pub fn for_render(edits: &Edits, frame: &RawFrame) -> Self {
        Self {
            render: RenderPlan::select(edits, frame),
            atmosphere: edits.basic.dehaze != 0.0,
        }
    }
}

pub(super) struct CachedFrame {
    pub texture: Arc<Texture>,
    pub width: u32,
    pub height: u32,
    pub block: usize,
}

pub(super) struct SensorCaches {
    pub frames: Mutex<lru::LruCache<u64, Arc<CachedFrame>>>,
    pub superpixels: Mutex<lru::LruCache<u64, Arc<CachedFrame>>>,
    pub atmospheres: Mutex<lru::LruCache<u64, [f32; 3]>>,
    pub atmosphere_estimates: AtomicU64,
    pub stages: StageCache,
}

impl SensorCaches {
    pub fn new(budget: Arc<GpuBudget>, texture_pool: Arc<TexturePool>) -> Self {
        let frames = NonZeroUsize::new(FRAME_CACHE_ITEMS).expect("nonzero");
        let atmospheres = NonZeroUsize::new(ATMOSPHERE_CACHE_ITEMS).expect("nonzero");
        Self {
            frames: Mutex::new(lru::LruCache::new(frames)),
            superpixels: Mutex::new(lru::LruCache::new(frames)),
            atmospheres: Mutex::new(lru::LruCache::new(atmospheres)),
            atmosphere_estimates: AtomicU64::new(0),
            stages: StageCache::new(budget, texture_pool),
        }
    }
}

impl GpuRenderer {
    pub fn atmosphere_estimates(&self) -> u64 {
        self.sensor.atmosphere_estimates.load(Ordering::Relaxed)
    }

    pub fn linear_source(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
    ) -> PipelineResult<LinearSource> {
        let timings = RenderTimings::new(&self.ctx);
        let edits = super::compose_edits(edits, options);
        let plan = SourcePlan::for_render(&edits, frame);
        self.sensor_stage(frame, &edits, options, plan, &timings, None)
    }

    pub fn render_source(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<RenderedSource> {
        if self.ctx.is_lost() {
            return Err(crate::PipelineError::DeviceLost);
        }
        let timings = RenderTimings::new(&self.ctx);
        let edits = super::compose_edits(edits, options);
        let linear = self.sensor_stage(
            frame,
            &edits,
            options,
            SourcePlan::PORTABLE,
            &timings,
            cancel,
        )?;
        let started = std::time::Instant::now();
        let image = self.read_source(&linear, cancel)?;
        timings
            .clock()
            .add_wall(timing::READBACK, started.elapsed());
        Ok(RenderedSource {
            image,
            renderer: "gpu".into(),
            timings: timings.finish(cancel),
        })
    }

    pub(super) fn sensor_stage(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        plan: SourcePlan,
        t: &RenderTimings,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<LinearSource> {
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
        let edits = edits.clamped();
        let cached = if edits.lens.any_active() {
            let corrected = t.stage(timing::LENS, || self.run_sensor(&cached, &edits))?;
            crate::cancel::check(cancel)?;
            corrected
        } else {
            cached
        };
        let dims = (cached.width, cached.height);
        if plan.render == RenderPlan::Fast {
            return Ok(LinearSource {
                meta: frame.meta.clone(),
                kind: LinearKind::PreWb,
                dims,
                texture: cached.texture.clone(),
                atmosphere: None,
            });
        }
        self.spatial_base(&cached, frame, &edits, options, plan.atmosphere, t, cancel)
    }

    #[allow(clippy::too_many_arguments)]
    fn spatial_base(
        &self,
        cached: &CachedFrame,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        with_atmosphere: bool,
        t: &RenderTimings,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<LinearSource> {
        let setup = crate::dcp_pipeline::resolve(&frame.meta, edits, options.dcp.as_deref());
        let dims = (cached.width, cached.height);
        let keys = StageKeys::new(frame, edits, dims, setup.cam_to_srgb);
        let wb_base = t.stage(timing::WB_PREPARE, || {
            self.run_wb_prepare(cached, &frame.meta, edits, &setup, keys.wb)
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
        let (spatial_dims, texture) = match preview_dims {
            Some(preview_dims) => {
                let downsampled = t.stage(timing::PREVIEW_RESAMPLE, || {
                    self.resample_lanczos(&full_src, dims, preview_dims, "preview-spatial-src")
                })?;
                crate::cancel::check(cancel)?;
                (preview_dims, downsampled)
            }
            None => (dims, full_src),
        };
        let atmosphere = if with_atmosphere {
            let key = keys.spatial(sigma, spatial_dims);
            let atm = t.stage(timing::DEHAZE, || {
                self.atmosphere_for(key, &texture, spatial_dims, cancel)
            })?;
            crate::cancel::check(cancel)?;
            Some(atm)
        } else {
            None
        };
        Ok(LinearSource {
            meta: frame.meta.clone(),
            kind: LinearKind::PostWb,
            dims: spatial_dims,
            texture,
            atmosphere,
        })
    }
}
