use std::time::Duration;

use wgpu::{Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, QUERY_SIZE};

use super::{GpuTimer, MAX_QUERIES, Pass, RenderTimings};
#[cfg(feature = "native")]
use crate::cancel::CancelToken;
use crate::gpu::readback::mapped_range;
use crate::timing::StageTiming;
use crate::{PipelineError, PipelineResult};

struct PendingResolve {
    readback: Buffer,
    passes: Vec<Pass>,
}

impl RenderTimings<'_> {
    #[cfg(feature = "native")]
    pub(in crate::gpu) fn finish(self, cancel: Option<&CancelToken>) -> Vec<StageTiming> {
        if let Some(gpu) = &self.gpu {
            warn_unresolved(self.resolve(gpu, cancel));
        }
        self.clock.finish()
    }

    #[cfg(feature = "web")]
    pub(in crate::gpu) async fn finish_async(self) -> Vec<StageTiming> {
        if let Some(gpu) = &self.gpu {
            warn_unresolved(self.resolve_async(gpu).await);
        }
        self.clock.finish()
    }

    #[cfg(feature = "native")]
    fn resolve(&self, gpu: &GpuTimer, cancel: Option<&CancelToken>) -> PipelineResult<()> {
        let Some(pending) = self.submit_resolve(gpu)? else {
            return Ok(());
        };
        crate::gpu::readback::map_buffer_cancellable(self.ctx, &pending.readback, cancel)?;
        self.record(pending)
    }

    #[cfg(feature = "web")]
    async fn resolve_async(&self, gpu: &GpuTimer) -> PipelineResult<()> {
        let Some(pending) = self.submit_resolve(gpu)? else {
            return Ok(());
        };
        crate::gpu::readback::map_read(&pending.readback).await?;
        self.record(pending)
    }

    fn submit_resolve(&self, gpu: &GpuTimer) -> PipelineResult<Option<PendingResolve>> {
        if *gpu.overflowed.lock() {
            return Err(PipelineError::Unsupported(format!(
                "render used more than {} timed compute passes",
                MAX_QUERIES / 2
            )));
        }
        let passes = std::mem::take(&mut *gpu.passes.lock());
        if passes.is_empty() {
            return Ok(None);
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
        Ok(Some(PendingResolve { readback, passes }))
    }

    fn record(&self, pending: PendingResolve) -> PipelineResult<()> {
        let ticks: Vec<u64> = {
            let slice = pending.readback.slice(..);
            let data = mapped_range(&slice)?;
            data.chunks_exact(QUERY_SIZE as usize)
                .map(|b| u64::from_le_bytes(b.try_into().unwrap_or_default()))
                .collect()
        };
        pending.readback.unmap();
        let period = f64::from(self.ctx.queue.get_timestamp_period());
        for pass in pending.passes {
            let begin = ticks[pass.begin as usize];
            let end = ticks[pass.begin as usize + 1];
            let nanos = (end.saturating_sub(begin) as f64 * period).round() as u64;
            self.clock.add_gpu(pass.stage, Duration::from_nanos(nanos));
        }
        Ok(())
    }
}

fn warn_unresolved(resolved: PipelineResult<()>) {
    match resolved {
        Ok(()) | Err(PipelineError::Cancelled) => {}
        Err(e) => tracing::warn!(error = %e, "gpu stage timestamps unavailable"),
    }
}
