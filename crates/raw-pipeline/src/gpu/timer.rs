use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::Mutex;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, ComputePassTimestampWrites, Device, QUERY_SIZE,
    QuerySet, QuerySetDescriptor, QueryType,
};

use super::context::GpuContext;
use crate::timing::StageClock;

mod resolve;

const MAX_QUERIES: u32 = 512;

pub(crate) type TimerSlots = Arc<Mutex<Vec<TimerSlot>>>;

pub(crate) struct TimerSlot {
    set: QuerySet,
    resolved: Buffer,
    readback: Buffer,
}

impl TimerSlot {
    fn new(device: &Device) -> Self {
        let size = u64::from(MAX_QUERIES) * u64::from(QUERY_SIZE);
        Self {
            set: device.create_query_set(&QuerySetDescriptor {
                label: Some("stage-timestamps"),
                ty: QueryType::Timestamp,
                count: MAX_QUERIES,
            }),
            resolved: device.create_buffer(&BufferDescriptor {
                label: Some("stage-timestamps-resolve"),
                size,
                usage: BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            readback: device.create_buffer(&BufferDescriptor {
                label: Some("stage-timestamps-readback"),
                size,
                usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
        }
    }
}

struct Pass {
    stage: &'static str,
    begin: u32,
}

struct GpuTimer {
    slot: Option<TimerSlot>,
    slots: TimerSlots,
    reusable: AtomicBool,
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

impl Drop for GpuTimer {
    fn drop(&mut self) {
        if !self.reusable.load(Ordering::Relaxed) {
            return;
        }
        if let Some(slot) = self.slot.take() {
            self.slots.lock().push(slot);
        }
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
    if let Some((timer, begin)) = &reserved
        && let Some(slot) = &timer.slot
    {
        return begin_pass(Some(ComputePassTimestampWrites {
            query_set: &slot.set,
            beginning_of_pass_write_index: Some(*begin),
            end_of_pass_write_index: Some(begin + 1),
        }));
    }
    begin_pass(None)
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
            let slot = ctx.timer_slots.lock().pop();
            Arc::new(GpuTimer {
                slot: Some(slot.unwrap_or_else(|| TimerSlot::new(&ctx.device))),
                slots: ctx.timer_slots.clone(),
                reusable: AtomicBool::new(true),
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
}

#[cfg(all(test, feature = "native"))]
mod tests {
    use super::*;

    fn timed_pass(ctx: &GpuContext, timings: &RenderTimings) {
        let _scope = timings.enter("probe");
        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        with_pass_timestamps(|writes| {
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: writes,
            });
        });
        ctx.queue.submit(Some(encoder.finish()));
    }

    #[test]
    fn renders_reuse_timestamp_slots() {
        let Ok(ctx) = GpuContext::with_timestamps(true) else {
            return;
        };
        if !ctx.timestamps {
            return;
        }
        for round in 0..8 {
            let timings = RenderTimings::new(&ctx);
            timed_pass(&ctx, &timings);
            let stages = timings.finish(None);
            if !stages.iter().any(|t| t.stage == "probe" && t.gpu.is_some()) {
                panic!("render {round} resolved no gpu time from a reused slot");
            }
        }
        drop([RenderTimings::new(&ctx), RenderTimings::new(&ctx)]);
        let slots = ctx.timer_slots.lock().len();
        if slots != 2 {
            panic!("{slots} timestamp slots after 8 sequential and 2 concurrent renders");
        }
    }
}
