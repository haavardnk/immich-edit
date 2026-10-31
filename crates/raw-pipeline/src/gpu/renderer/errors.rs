use wgpu::{Error, ErrorFilter};

use super::GpuRenderer;
use crate::{PipelineError, PipelineResult};

#[cfg(all(test, feature = "native"))]
mod tests;

impl GpuRenderer {
    pub async fn guarded<T>(
        &self,
        work: impl Future<Output = PipelineResult<T>>,
    ) -> PipelineResult<T> {
        if self.ctx.is_lost() {
            return Err(PipelineError::DeviceLost);
        }
        let device = &self.ctx.device;
        let out_of_memory = device.push_error_scope(ErrorFilter::OutOfMemory);
        let validation = device.push_error_scope(ErrorFilter::Validation);
        let result = work.await;
        let invalid = validation.pop().await;
        let exhausted = out_of_memory.pop().await;
        #[cfg(feature = "native")]
        if exhausted.is_some() {
            self.admission.lower(self.retention.device_bytes());
        }
        let Some(error) = exhausted.or(invalid) else {
            return result;
        };
        self.discard_resources();
        match result {
            Err(PipelineError::Cancelled) => result,
            _ => Err(pipeline_error(error)),
        }
    }

    fn discard_resources(&self) {
        #[cfg(feature = "native")]
        self.sensor.clear();
        self.output_pool.lock().clear();
        self.sharpen_pool.lock().clear();
        self.atlas_pool.lock().clear();
        self.lut_tex_cache.lock().clear();
        self.huesat_tex_cache.lock().clear();
        self.depth_tex_cache.lock().clear();
        self.texture_pool.trim(u64::MAX);
        self.uniform_pool.clear();
        self.ctx.timer_slots.lock().clear();
    }
}

fn pipeline_error(error: Error) -> PipelineError {
    match error {
        Error::OutOfMemory { source } => PipelineError::OutOfMemory(source.to_string()),
        Error::Validation { description, .. } | Error::Internal { description, .. } => {
            PipelineError::Render(format!("gpu: {description}"))
        }
    }
}
