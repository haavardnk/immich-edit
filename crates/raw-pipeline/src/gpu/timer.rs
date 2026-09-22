use std::cell::RefCell;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use wgpu::{
    BufferDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePassTimestampWrites,
    QUERY_SIZE, QuerySet, QuerySetDescriptor, QueryType,
};

use super::context::GpuContext;
use super::readback::{map_buffer_cancellable, mapped_range};
use crate::cancel::CancelToken;
use crate::timing::{StageClock, StageTiming};
use crate::{PipelineError, PipelineResult};

const MAX_QUERIES: u32 = 512;

struct Pass {
    stage: &'static str,
    begin: u32,
}

struct GpuTimer {
    set: QuerySet,
    passes: Mutex<Vec<Pass>>,
    overflowed: Mutex<bool>,
}

impl GpuTimer {
    fn reserve(&self, stage: &'static str) -> Option<u32> {
        let mut passes = self.passes.lock();
        let begin = passes.len() as u32 * 2;
        if begin + 2 > MAX_QUERIES {
            *self.overflowed.lock() = true;
            return None;
        }
        passes.push(Pass { stage, begin });
        Some(begin)
    }
}

thread_local! {
    static ACTIVE: RefCell<Option<(Arc<GpuTimer>, &'static str)>> = const { RefCell::new(None) };
}

pub(super) fn with_pass_timestamps<R>(
    begin_pass: impl FnOnce(Option<ComputePassTimestampWrites<'_>>) -> R,
) -> R {
    let active = ACTIVE.with(|a| a.borrow().clone());
    let reserved = active.and_then(|(timer, stage)| timer.reserve(stage).map(|i| (timer, i)));
    match &reserved {
        Some((timer, begin)) => begin_pass(Some(ComputePassTimestampWrites {
            query_set: &timer.set,
            beginning_of_pass_write_index: Some(*begin),
            end_of_pass_write_index: Some(begin + 1),
        })),
        None => begin_pass(None),
    }
}

pub(super) struct StageScope {
    previous: Option<(Arc<GpuTimer>, &'static str)>,
}

impl Drop for StageScope {
    fn drop(&mut self) {
        let previous = self.previous.take();
        ACTIVE.with(|a| *a.borrow_mut() = previous);
    }
}

pub(super) struct RenderTimings<'a> {
    ctx: &'a GpuContext,
    clock: StageClock,
    gpu: Option<Arc<GpuTimer>>,
}

impl<'a> RenderTimings<'a> {
    pub(super) fn new(ctx: &'a GpuContext) -> Self {
        let gpu = ctx.timestamps.then(|| {
            Arc::new(GpuTimer {
                set: ctx.device.create_query_set(&QuerySetDescriptor {
                    label: Some("stage-timestamps"),
                    ty: QueryType::Timestamp,
                    count: MAX_QUERIES,
                }),
                passes: Mutex::new(Vec::new()),
                overflowed: Mutex::new(false),
            })
        });
        Self {
            ctx,
            clock: StageClock::default(),
            gpu,
        }
    }

    pub(super) fn clock(&self) -> &StageClock {
        &self.clock
    }

    pub(super) fn enter(&self, stage: &'static str) -> StageScope {
        let next = self.gpu.clone().map(|timer| (timer, stage));
        let previous = ACTIVE.with(|a| std::mem::replace(&mut *a.borrow_mut(), next));
        StageScope { previous }
    }

    pub(super) fn stage<T>(&self, stage: &'static str, f: impl FnOnce() -> T) -> T {
        let _scope = self.enter(stage);
        self.clock.time(stage, f)
    }

    pub(super) fn finish(self, cancel: Option<&CancelToken>) -> Vec<StageTiming> {
        if let Some(gpu) = &self.gpu {
            match self.resolve(gpu, cancel) {
                Ok(()) | Err(PipelineError::Cancelled) => {}
                Err(e) => tracing::warn!(error = %e, "gpu stage timestamps unavailable"),
            }
        }
        self.clock.finish()
    }

    fn resolve(&self, gpu: &GpuTimer, cancel: Option<&CancelToken>) -> PipelineResult<()> {
        if *gpu.overflowed.lock() {
            return Err(PipelineError::Unsupported(format!(
                "render used more than {} timed compute passes",
                MAX_QUERIES / 2
            )));
        }
        let passes = std::mem::take(&mut *gpu.passes.lock());
        if passes.is_empty() {
            return Ok(());
        }
        let count = passes.len() as u32 * 2;
        let size = u64::from(count) * u64::from(QUERY_SIZE);
        let device = &self.ctx.device;
        let resolved = device.create_buffer(&BufferDescriptor {
            label: Some("stage-timestamps-resolve"),
            size,
            usage: BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&BufferDescriptor {
            label: Some("stage-timestamps-readback"),
            size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("stage-timestamps-resolve-enc"),
        });
        encoder.resolve_query_set(&gpu.set, 0..count, &resolved, 0);
        encoder.copy_buffer_to_buffer(&resolved, 0, &readback, 0, size);
        self.ctx.queue.submit(Some(encoder.finish()));
        map_buffer_cancellable(self.ctx, &readback, cancel)?;
        let ticks: Vec<u64> = {
            let slice = readback.slice(..);
            let data = mapped_range(&slice)?;
            data.chunks_exact(QUERY_SIZE as usize)
                .map(|b| u64::from_le_bytes(b.try_into().unwrap_or_default()))
                .collect()
        };
        readback.unmap();
        let period = f64::from(self.ctx.queue.get_timestamp_period());
        for pass in passes {
            let begin = ticks[pass.begin as usize];
            let end = ticks[pass.begin as usize + 1];
            let nanos = (end.saturating_sub(begin) as f64 * period).round() as u64;
            self.clock.add_gpu(pass.stage, Duration::from_nanos(nanos));
        }
        Ok(())
    }
}
