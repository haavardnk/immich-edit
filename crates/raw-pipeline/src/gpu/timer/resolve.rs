use std::sync::atomic::Ordering;
use std::time::Duration;

use wgpu::{CommandEncoderDescriptor, QUERY_SIZE};

use super::{GpuTimer, MAX_QUERIES, Pass, RenderTimings, TimerSlot};
#[cfg(feature = "native")]
use crate::cancel::CancelToken;
use crate::gpu::readback::mapped_range;
use crate::timing::StageTiming;
use crate::{PipelineError, PipelineResult};

impl RenderTimings<'_> {
    #[cfg(feature = "native")]
    pub(in crate::gpu) fn finish(self, cancel: Option<&CancelToken>) -> Vec<StageTiming> {
        if let Some(gpu) = &self.gpu {
            settle(gpu, self.resolve(gpu, cancel));
        }
        self.clock.finish()
    }

    #[cfg(feature = "web")]
    pub(in crate::gpu) async fn finish_async(self) -> Vec<StageTiming> {
        if let Some(gpu) = &self.gpu {
            settle(gpu, self.resolve_async(gpu).await);
        }
        self.clock.finish()
    }

    #[cfg(feature = "native")]
    fn resolve(&self, gpu: &GpuTimer, cancel: Option<&CancelToken>) -> PipelineResult<()> {
        let Some((slot, passes)) = self.submit_resolve(gpu)? else {
            return Ok(());
        };
        crate::gpu::readback::map_buffer_cancellable(self.ctx, &slot.readback, cancel)?;
        self.record(slot, passes)
    }

    #[cfg(feature = "web")]
    async fn resolve_async(&self, gpu: &GpuTimer) -> PipelineResult<()> {
        let Some((slot, passes)) = self.submit_resolve(gpu)? else {
            return Ok(());
        };
        crate::gpu::readback::map_read(&slot.readback).await?;
        self.record(slot, passes)
    }

    fn submit_resolve<'g>(
        &self,
        gpu: &'g GpuTimer,
    ) -> PipelineResult<Option<(&'g TimerSlot, Vec<Pass>)>> {
        if *gpu.overflowed.lock() {
            return Err(PipelineError::Unsupported(format!(
                "render used more than {} timed compute passes",
                MAX_QUERIES / 2
            )));
        }
        let passes = std::mem::take(&mut *gpu.passes.lock());
        let Some(slot) = &gpu.slot else {
            return Ok(None);
        };
        if passes.is_empty() {
            return Ok(None);
        }
        let count = passes.len() as u32 * 2;
        let size = u64::from(count) * u64::from(QUERY_SIZE);
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("stage-timestamps-resolve-enc"),
            });
        encoder.resolve_query_set(&slot.set, 0..count, &slot.resolved, 0);
        encoder.copy_buffer_to_buffer(&slot.resolved, 0, &slot.readback, 0, size);
        self.ctx.queue.submit(Some(encoder.finish()));
        Ok(Some((slot, passes)))
    }

    fn record(&self, slot: &TimerSlot, passes: Vec<Pass>) -> PipelineResult<()> {
        let size = passes.len() as u64 * 2 * u64::from(QUERY_SIZE);
        let ticks: Vec<u64> = {
            let slice = slot.readback.slice(..size);
            let data = mapped_range(&slice)?;
            data.chunks_exact(QUERY_SIZE as usize)
                .map(|b| u64::from_le_bytes(b.try_into().unwrap_or_default()))
                .collect()
        };
        slot.readback.unmap();
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

fn settle(gpu: &GpuTimer, resolved: PipelineResult<()>) {
    match resolved {
        Ok(()) | Err(PipelineError::Cancelled) => {}
        Err(e) => {
            gpu.reusable.store(false, Ordering::Relaxed);
            tracing::warn!(error = %e, "gpu stage timestamps unavailable");
        }
    }
}
