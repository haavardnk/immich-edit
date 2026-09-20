use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use raw_pipeline::CancelToken;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{RawFrame, RenderOptions};
use raw_pipeline::{CpuRenderer, GpuRenderer, GpuRendererOptions, PipelineError, RenderedImage};

use crate::config::RendererMode;

const GPU_REBUILD_MIN_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveRenderer {
    Cpu,
    Gpu,
}

impl ActiveRenderer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Gpu => "gpu",
        }
    }
}

#[derive(Clone)]
pub struct RenderDevice {
    gpu: Arc<RwLock<Option<Arc<GpuRenderer>>>>,
    cpu: Arc<CpuRenderer>,
    mode: RendererMode,
    texture_cache_bytes: u64,
    active: Arc<RwLock<ActiveRenderer>>,
    label: Arc<RwLock<Option<String>>>,
    last_rebuild: Arc<RwLock<Option<Instant>>>,
}

impl RenderDevice {
    pub fn new(mode: RendererMode, texture_cache_bytes: u64) -> Self {
        let (gpu, active, label) = init_gpu(mode, texture_cache_bytes);
        Self {
            gpu: Arc::new(RwLock::new(gpu)),
            cpu: Arc::new(CpuRenderer::new()),
            mode,
            texture_cache_bytes,
            active: Arc::new(RwLock::new(active)),
            label: Arc::new(RwLock::new(label)),
            last_rebuild: Arc::new(RwLock::new(None)),
        }
    }

    pub fn active(&self) -> ActiveRenderer {
        *self.active.read().unwrap()
    }

    pub fn label(&self) -> Option<String> {
        self.label.read().unwrap().clone()
    }

    pub fn pool_stats(&self) -> Option<raw_pipeline::GpuPoolStats> {
        self.gpu.read().unwrap().as_ref().map(|g| g.pool_stats())
    }

    pub fn render_blocking(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        opts: &RenderOptions,
        cancel: Option<&CancelToken>,
    ) -> Result<RenderedImage, PipelineError> {
        tracing::debug!(
            orient = ?frame.orientation,
            sensor_w = frame.width,
            sensor_h = frame.height,
            "render orientation"
        );
        if matches!(self.mode, RendererMode::Cpu) {
            return self.cpu.render_with_cancel(frame, edits, opts, cancel);
        }
        let gpu = self.gpu_or_rebuild();
        if let Some(g) = gpu {
            if g.is_lost() {
                self.handle_device_lost();
            } else {
                match g.render_with_cancel(frame, edits, opts, cancel) {
                    Ok(r) => return Ok(r),
                    Err(PipelineError::Cancelled) => return Err(PipelineError::Cancelled),
                    Err(PipelineError::DeviceLost) => {
                        self.handle_device_lost();
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "gpu render failed; falling back to cpu");
                    }
                }
            }
        }
        self.cpu.render_with_cancel(frame, edits, opts, cancel)
    }

    fn gpu_or_rebuild(&self) -> Option<Arc<GpuRenderer>> {
        if let Some(g) = self.gpu.read().unwrap().clone() {
            return Some(g);
        }
        let mut last = self.last_rebuild.write().unwrap();
        let now = Instant::now();
        if let Some(t) = *last
            && now.duration_since(t) < GPU_REBUILD_MIN_INTERVAL
        {
            return None;
        }
        *last = Some(now);
        drop(last);
        match GpuRenderer::with_options(GpuRendererOptions {
            texture_pool_max_bytes: self.texture_cache_bytes,
        }) {
            Ok(r) => {
                let label = r.adapter_label();
                tracing::info!(adapter = %label, "gpu renderer rebuilt after device loss");
                let arc = Arc::new(r);
                *self.gpu.write().unwrap() = Some(arc.clone());
                *self.label.write().unwrap() = Some(label);
                *self.active.write().unwrap() = ActiveRenderer::Gpu;
                Some(arc)
            }
            Err(e) => {
                tracing::warn!(error = %e, "gpu rebuild failed; staying on cpu");
                None
            }
        }
    }

    fn handle_device_lost(&self) {
        tracing::error!("gpu device lost; dropping renderer and falling back to cpu");
        *self.gpu.write().unwrap() = None;
        *self.label.write().unwrap() = None;
        *self.active.write().unwrap() = ActiveRenderer::Cpu;
        *self.last_rebuild.write().unwrap() = Some(Instant::now());
    }
}

fn init_gpu(
    mode: RendererMode,
    texture_pool_max_bytes: u64,
) -> (Option<Arc<GpuRenderer>>, ActiveRenderer, Option<String>) {
    if matches!(mode, RendererMode::Cpu) {
        return (None, ActiveRenderer::Cpu, None);
    }
    match GpuRenderer::with_options(GpuRendererOptions {
        texture_pool_max_bytes,
    }) {
        Ok(r) => {
            let label = r.adapter_label();
            tracing::info!(adapter = %label, "gpu renderer initialized");
            (Some(Arc::new(r)), ActiveRenderer::Gpu, Some(label))
        }
        Err(e) => {
            if matches!(mode, RendererMode::Gpu) {
                tracing::error!(error = %e, "gpu requested but unavailable; falling back to cpu");
            } else {
                tracing::warn!(error = %e, "gpu unavailable; using cpu");
            }
            (None, ActiveRenderer::Cpu, None)
        }
    }
}
