mod shape;
mod tile;
mod tiled;

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::Mutex;
use wgpu::Texture;

use super::cache_keys::StageKeys;
use super::detail::NoiseProfile;
use super::stage_cache::StageCache;
use super::{GpuRenderer, RenderPlan};
use crate::PipelineResult;
use crate::ai_denoise::{DenoiseMix, DenoisePlane};
use crate::cancel::CancelToken;
use crate::dcp::setup::DcpSetup;
use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions};
use crate::gpu::budget::GpuBudget;
use crate::gpu::source::LinearSource;
use crate::gpu::texture_pool::TexturePool;
use crate::gpu::timer::RenderTimings;
use crate::source::{LinearKind, RenderedSource, SourceWindow};
use crate::tile::{Rect, TilePlan};
use crate::timing;
pub(super) use shape::SensorShape;

const FRAME_CACHE_ITEMS: usize = 2;
const ATMOSPHERE_CACHE_ITEMS: usize = 16;
const NOISE_PROFILE_CACHE_ITEMS: usize = 16;

#[derive(Clone, Copy)]
pub(super) struct SourcePlan {
    render: RenderPlan,
    atmosphere: bool,
    display: bool,
}

impl SourcePlan {
    const PORTABLE: Self = Self {
        render: RenderPlan::Presence,
        atmosphere: true,
        display: false,
    };

    pub fn for_render(edits: &Edits, frame: &RawFrame) -> Self {
        Self {
            render: RenderPlan::select(edits, frame),
            atmosphere: edits.basic.dehaze != 0.0,
            display: true,
        }
    }

    pub fn presence(&self) -> bool {
        self.render == RenderPlan::Presence
    }

    pub fn displays(&self) -> bool {
        self.display
    }
}

pub(super) struct SensorFrame {
    pub texture: Arc<Texture>,
    pub rect: Rect,
    pub block: usize,
}

struct DetailJob {
    setup: DcpSetup,
    keys: StageKeys,
}

pub(super) struct SensorJob<'a> {
    frame: &'a RawFrame,
    edits: Edits,
    plan: SourcePlan,
    tiles: TilePlan,
    shape: SensorShape,
    denoise: Option<(&'a DenoisePlane, DenoiseMix)>,
    detail: Option<DetailJob>,
    t: &'a RenderTimings<'a>,
    cancel: Option<&'a CancelToken>,
}

impl<'a> SensorJob<'a> {
    pub fn new(
        frame: &'a RawFrame,
        edits: &Edits,
        options: &'a RenderOptions,
        plan: SourcePlan,
        tiles: TilePlan,
        t: &'a RenderTimings<'a>,
        cancel: Option<&'a CancelToken>,
    ) -> PipelineResult<Self> {
        let edits = edits.clamped();
        let shape = SensorShape::new(frame, &edits, options, plan.presence());
        let denoise = crate::ai_denoise::active_plane(frame, &edits, options)?
            .map(|(plane, mix)| (plane.as_ref(), mix));
        let detail = plan.presence().then(|| {
            let setup = crate::dcp::setup::resolve(&frame.meta, &edits, options.dcp.as_deref());
            let keys = StageKeys::new(
                frame,
                &edits,
                shape.sensor(),
                setup.cam_to_srgb,
                denoise.map(|(plane, _)| plane),
            );
            DetailJob { setup, keys }
        });
        Ok(Self {
            frame,
            edits,
            plan,
            tiles,
            shape,
            denoise,
            detail,
            t,
            cancel,
        })
    }

    pub fn shape(&self) -> &SensorShape {
        &self.shape
    }

    fn working_mips(&self) -> bool {
        !self.shape.resamples() || self.plan.atmosphere
    }

    fn cached(&self) -> bool {
        self.tiles == TilePlan::SinglePass
    }

    fn cache(&self, key: u64) -> Option<u64> {
        self.cached().then_some(key)
    }

    fn check(&self) -> PipelineResult<()> {
        crate::cancel::check(self.cancel)
    }

    fn nr_active(&self) -> bool {
        let d = &self.edits.detail;
        d.luma_nr_active() || d.color_nr_active()
    }

    fn source(&self, texture: Arc<Texture>, atmosphere: Option<[f32; 3]>) -> LinearSource {
        let kind = if self.plan.presence() {
            LinearKind::PostWb
        } else {
            LinearKind::PreWb
        };
        LinearSource {
            meta: self.frame.meta.clone(),
            kind,
            dims: self.shape.working(),
            texture,
            atmosphere,
            window: None,
        }
    }

    fn window_source(&self, texture: Arc<Texture>, rect: Rect) -> LinearSource {
        LinearSource {
            dims: rect.dims(),
            window: Some(SourceWindow {
                origin: (rect.x, rect.y),
                full: self.shape.working(),
            }),
            ..self.source(texture, None)
        }
    }
}

pub(super) struct SensorCaches {
    pub frames: Mutex<lru::LruCache<u64, Arc<SensorFrame>>>,
    pub superpixels: Mutex<lru::LruCache<u64, Arc<SensorFrame>>>,
    pub denoise_blends: AtomicU64,
    pub atmospheres: Mutex<lru::LruCache<u64, [f32; 3]>>,
    pub atmosphere_estimates: AtomicU64,
    pub noise_profiles: Mutex<lru::LruCache<u64, NoiseProfile>>,
    pub noise_estimates: AtomicU64,
    pub tiles: AtomicU64,
    pub stages: StageCache,
}

impl SensorCaches {
    pub fn new(budget: Arc<GpuBudget>, texture_pool: Arc<TexturePool>) -> Self {
        let frames = NonZeroUsize::new(FRAME_CACHE_ITEMS).expect("nonzero");
        let atmospheres = NonZeroUsize::new(ATMOSPHERE_CACHE_ITEMS).expect("nonzero");
        let noise_profiles = NonZeroUsize::new(NOISE_PROFILE_CACHE_ITEMS).expect("nonzero");
        Self {
            frames: Mutex::new(lru::LruCache::new(frames)),
            superpixels: Mutex::new(lru::LruCache::new(frames)),
            denoise_blends: AtomicU64::new(0),
            atmospheres: Mutex::new(lru::LruCache::new(atmospheres)),
            atmosphere_estimates: AtomicU64::new(0),
            noise_profiles: Mutex::new(lru::LruCache::new(noise_profiles)),
            noise_estimates: AtomicU64::new(0),
            tiles: AtomicU64::new(0),
            stages: StageCache::new(budget, texture_pool),
        }
    }

    pub fn clear(&self) {
        self.frames.lock().clear();
        self.superpixels.lock().clear();
        self.atmospheres.lock().clear();
        self.noise_profiles.lock().clear();
        self.stages.clear();
    }
}

impl GpuRenderer {
    pub fn atmosphere_estimates(&self) -> u64 {
        self.sensor.atmosphere_estimates.load(Ordering::Relaxed)
    }

    pub fn noise_estimates(&self) -> u64 {
        self.sensor.noise_estimates.load(Ordering::Relaxed)
    }

    pub fn sensor_tiles(&self) -> u64 {
        self.sensor.tiles.load(Ordering::Relaxed)
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
        let tiles = TilePlan::SinglePass;
        self.sensor_stage(&SensorJob::new(
            frame, &edits, options, plan, tiles, &timings, None,
        )?)
    }

    pub fn render_source(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<RenderedSource> {
        let composed = super::compose_edits(edits, options);
        self.run_planned(
            Some((frame, SourcePlan::PORTABLE)),
            &composed,
            options,
            |tiles| self.estimate_source(frame, edits, options, tiles),
            |tiles| self.render_source_planned(frame, &composed, options, tiles, cancel),
            cancel,
        )
    }

    fn render_source_planned(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        tiles: TilePlan,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<RenderedSource> {
        pollster::block_on(self.guarded(async {
            let timings = RenderTimings::new(&self.ctx);
            let plan = SourcePlan::PORTABLE;
            let job = SensorJob::new(frame, edits, options, plan, tiles, &timings, cancel)?;
            let linear = self.sensor_stage(&job)?;
            let started = std::time::Instant::now();
            let window = options
                .roi
                .and_then(|_| crate::source::window_rect(&linear.meta, edits, linear.dims));
            let image = self.read_source(&linear, window, cancel)?;
            timings
                .clock()
                .add_wall(timing::READBACK, started.elapsed());
            Ok(RenderedSource {
                image,
                renderer: "gpu".into(),
                timings: timings.finish(cancel),
            })
        }))
    }

    pub(super) fn sensor_stage(&self, job: &SensorJob<'_>) -> PipelineResult<LinearSource> {
        match job.tiles {
            TilePlan::SensorTiled { edge } => self.sensor_tiled(job, edge),
            TilePlan::SinglePass | TilePlan::DisplayTiled { .. } => self.sensor_single(job),
        }
    }

    fn sensor_single(&self, job: &SensorJob<'_>) -> PipelineResult<LinearSource> {
        let tile = job.shape.chain.whole();
        let base = self.sensor_base(job, &tile)?;
        let Some(detail) = &job.detail else {
            return Ok(job.source(base.texture.clone(), None));
        };
        let wb = self.sensor_wb(job, detail, &base)?;
        let (texture, _) = self.sensor_detail(job, detail, &tile, &wb, None)?;
        let atmosphere = self.atmosphere(job, detail, &texture)?;
        Ok(job.source(texture, atmosphere))
    }

    fn atmosphere(
        &self,
        job: &SensorJob<'_>,
        detail: &DetailJob,
        texture: &Texture,
    ) -> PipelineResult<Option<[f32; 3]>> {
        if !job.plan.atmosphere {
            return Ok(None);
        }
        let working = job.shape.working();
        let key = detail.keys.spatial(job.shape.sigma, working);
        let atmosphere = job.t.stage(timing::DEHAZE, || {
            self.atmosphere_for(key, texture, working, job.cancel)
        })?;
        job.check()?;
        Ok(Some(atmosphere))
    }
}
