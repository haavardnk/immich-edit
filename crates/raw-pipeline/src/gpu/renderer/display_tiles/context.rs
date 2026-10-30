mod plan;

use wgpu::{CommandEncoderDescriptor, TextureUsages};

use super::super::GpuRenderer;
use super::super::detail::{HazeSample, NoiseProfile, PRESENCE_PYRAMID, PyramidSource, level_dims};
use super::super::display::{GlobalContext, GridView, HazeGrid, LevelView, StageState};
use super::super::sensor_stage::SensorJob;
use super::DisplayTiles;
use crate::frame::RenderOptions;
use crate::gpu::dispatch::{Region, copy_region};
use crate::gpu::helpers::mip_count;
use crate::gpu::source::LinearSource;
use crate::gpu::texture::{STORAGE_SAMPLED, full_view};
use crate::gpu::texture_pool::{PooledTexture, TextureKey};
use crate::tile::Rect;
use crate::{PipelineError, PipelineResult, timing};
pub(in crate::gpu::renderer) use plan::{Coarse, ContextPlan};

const LEVEL_USAGE: TextureUsages = STORAGE_SAMPLED.union(TextureUsages::COPY_DST);
const CORE_USAGE: TextureUsages = LEVEL_USAGE.union(TextureUsages::COPY_SRC);

pub(super) struct TileContext {
    pub context: GlobalContext,
    _held: Vec<PooledTexture>,
}

pub(super) struct ContextJob<'j> {
    pub sensor: &'j SensorJob<'j>,
    pub options: &'j RenderOptions,
    pub profile: Option<NoiseProfile>,
    pub plan: &'j ContextPlan,
    pub layout: &'j DisplayTiles,
}

fn level_range(core: Rect, level: u32) -> Option<Rect> {
    let range = Rect::bounds(
        core.x >> level,
        core.y >> level,
        core.right() >> level,
        core.bottom() >> level,
    );
    (range.w > 0 && range.h > 0).then_some(range)
}

fn texel_origin(linear: &LinearSource, rect: Rect) -> (u32, u32) {
    let (x, y) = linear.origin();
    (rect.x - x, rect.y - y)
}

impl GpuRenderer {
    pub(super) fn build_context(&self, job: &ContextJob<'_>) -> PipelineResult<TileContext> {
        let started = std::time::Instant::now();
        let plan = job.plan;
        let mut held = Vec::new();
        let mut context = GlobalContext::default();
        if plan.needs.dehaze {
            let (haze, ab) = self.build_haze(job)?;
            context.haze = Some(haze);
            held.push(ab);
        }
        for adjusted in [false, true] {
            let coarse = plan.collected(adjusted);
            if coarse.is_empty() {
                continue;
            }
            let levels = self.collect_levels(job, &context, adjusted, &coarse)?;
            held.extend(self.finish_levels(&mut context, plan, coarse.into_iter().zip(levels)));
        }
        tracing::debug!(
            tiles = job.layout.cores.len(),
            ms = started.elapsed().as_millis() as u64,
            "gpu display tile context"
        );
        Ok(TileContext {
            context,
            _held: held,
        })
    }

    fn acquire_context(
        &self,
        dims: (u32, u32),
        levels: u32,
        usage: TextureUsages,
    ) -> PooledTexture {
        let key = TextureKey::new(self.ctx.linear_format, dims.0, dims.1, levels, usage);
        self.texture_pool
            .acquire(&self.ctx.device, key, "tile-context")
    }

    fn build_haze(&self, job: &ContextJob<'_>) -> PipelineResult<(HazeGrid, PooledTexture)> {
        let (sensor, plan) = (job.sensor, job.plan);
        let key = sensor.atmosphere_key().ok_or_else(|| {
            PipelineError::Unsupported("dehaze needs an atmosphere estimate on the source".into())
        })?;
        let lo_src = self.acquire_context(plan.lo(), 1, STORAGE_SAMPLED);
        let atmosphere = match self.cached_atmosphere(key) {
            Some(atmosphere) => {
                self.collect_haze(job, &lo_src, None)?;
                atmosphere
            }
            None => {
                let dims = plan.level_dims(plan.atmosphere);
                let levels = self.acquire_context(dims, 1, CORE_USAGE);
                self.collect_haze(job, &lo_src, Some(&levels))?;
                sensor.timings().stage(timing::DEHAZE, || {
                    self.atmosphere_for(key, || {
                        self.read_atmosphere(&levels, 0, dims, sensor.cancel())
                    })
                })?
            }
        };
        let ab = self.submit_haze_grid(&lo_src, plan.lo(), plan.haze, atmosphere);
        let haze = HazeGrid {
            atmosphere,
            ab: GridView::global(full_view(&ab)),
        };
        Ok((haze, ab))
    }

    fn collect_haze(
        &self,
        job: &ContextJob<'_>,
        lo_src: &PooledTexture,
        levels: Option<&PooledTexture>,
    ) -> PipelineResult<()> {
        let (sensor, plan) = (job.sensor, job.plan);
        let scale = plan.haze.scale;
        let pooled = self.texture_pool.bytes();
        for &core in &job.layout.cores {
            let linear = self.sensor_window(sensor, core, job.profile)?;
            if let Some(levels) = levels {
                self.place_atmosphere(&linear, core, plan.atmosphere, levels);
            }
            if let Some(cells) = level_range(core, scale.trailing_zeros()) {
                let read = texel_origin(&linear, core);
                let sample = HazeSample { cells, read, scale };
                self.submit_haze_downsample(&linear.texture, lo_src, sample);
            }
            self.wait_idle(pooled)?;
            sensor.check()?;
        }
        Ok(())
    }

    fn place_atmosphere(
        &self,
        linear: &LinearSource,
        core: Rect,
        level: u32,
        levels: &PooledTexture,
    ) {
        let Some(range) = level_range(core, level) else {
            return;
        };
        let rgb = self.acquire_context(core.dims(), mip_count(core.w, core.h), CORE_USAGE);
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("tile-context-atmosphere"),
            });
        copy_region(
            &mut encoder,
            Region::at(&linear.texture, 0, texel_origin(linear, core)),
            Region::at(&rgb, 0, (0, 0)),
            core.dims(),
        );
        self.encode_mipgen(&mut encoder, &rgb, core.w, core.h);
        copy_region(
            &mut encoder,
            Region::at(&rgb, level, (0, 0)),
            Region::at(levels, 0, (range.x, range.y)),
            range.dims(),
        );
        self.ctx.queue.submit(Some(encoder.finish()));
    }

    fn collect_levels(
        &self,
        job: &ContextJob<'_>,
        context: &GlobalContext,
        adjusted: bool,
        coarse: &[Coarse],
    ) -> PipelineResult<Vec<PooledTexture>> {
        let (sensor, plan) = (job.sensor, job.plan);
        let levels: Vec<u32> = coarse.iter().map(|&c| plan.level(c)).collect();
        let targets: Vec<PooledTexture> = levels
            .iter()
            .map(|&level| self.acquire_context(plan.level_dims(level), 1, LEVEL_USAGE))
            .collect();
        let (edits, t, cancel) = (sensor.edits(), sensor.timings(), sensor.cancel());
        let meta = &sensor.frame().meta;
        let pooled = self.texture_pool.bytes();
        for &core in &job.layout.cores {
            let window = job.layout.context_window(core);
            let linear = self.sensor_window(sensor, window, job.profile)?;
            let mut state = StageState::new(&linear);
            self.dehaze_stage(&mut state, edits, Some(context), t, cancel)?;
            self.lens_blur_stage(&mut state, meta, edits, job.options, t, cancel)?;
            if adjusted {
                self.presence_stage(&mut state, edits, &context.presence, t, cancel)?;
            }
            self.place_levels(&state, core, &levels, &targets);
            drop(state);
            self.wait_idle(pooled)?;
            sensor.check()?;
        }
        Ok(targets)
    }

    fn place_levels(
        &self,
        state: &StageState,
        core: Rect,
        levels: &[u32],
        targets: &[PooledTexture],
    ) {
        let top = levels.iter().max().map_or(1, |level| level + 1);
        let count = top.min(mip_count(core.w, core.h));
        let pyramid = self.acquire_context(core.dims(), count, CORE_USAGE);
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("tile-context-levels"),
            });
        let source = PyramidSource {
            view: &full_view(&state.texture),
            offset: (core.x - state.origin.0, core.y - state.origin.1),
            dims: core.dims(),
        };
        let _pyramid =
            self.encode_luma_pyramid(&mut encoder, source, &pyramid, count, &PRESENCE_PYRAMID);
        for (&level, target) in levels.iter().zip(targets) {
            if let Some(range) = level_range(core, level) {
                copy_region(
                    &mut encoder,
                    Region::at(&pyramid, level, (0, 0)),
                    Region::at(target, 0, (range.x, range.y)),
                    range.dims(),
                );
            }
        }
        self.ctx.queue.submit(Some(encoder.finish()));
    }

    fn finish_levels(
        &self,
        context: &mut GlobalContext,
        plan: &ContextPlan,
        levels: impl Iterator<Item = (Coarse, PooledTexture)>,
    ) -> Vec<PooledTexture> {
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("tile-context-finish"),
            });
        let mut held = Vec::new();
        let mut guides = Vec::new();
        let mut blurs = Vec::new();
        let mut spent = Vec::new();
        for (coarse, texture) in levels {
            let level = plan.level(coarse);
            let dims = level_dims(plan.working, level);
            let view = full_view(&texture);
            let global = |view| LevelView {
                grid: GridView::global(view),
                level,
            };
            match coarse {
                Coarse::Clarity => {
                    let guide =
                        self.encode_clarity_guide(&mut encoder, &view, dims, plan.blurs.clarity);
                    context.presence.guide = Some(global(full_view(&guide.texture)));
                    guides.push(guide);
                    spent.push(texture);
                }
                Coarse::Texture => {
                    let blur = plan.blurs.texture;
                    blurs.push(self.encode_level_blur(&mut encoder, &view, dims, blur));
                    context.presence.texture = Some(global(view));
                    held.push(texture);
                }
                Coarse::Shadows => {
                    let blur = plan.blurs.shadows;
                    blurs.push(self.encode_level_blur(&mut encoder, &view, dims, blur));
                    context.shadows = Some(global(view));
                    held.push(texture);
                }
            }
        }
        self.ctx.queue.submit(Some(encoder.finish()));
        held.extend(guides.into_iter().map(|guide| guide.texture));
        held
    }
}
