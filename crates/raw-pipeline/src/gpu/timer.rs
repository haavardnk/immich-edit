use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::Mutex;
use wgpu::{ComputePassTimestampWrites, QuerySet, QuerySetDescriptor, QueryType};

use super::context::GpuContext;
use crate::timing::StageClock;

mod resolve;

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
}
