use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use raw_pipeline::CancelToken;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{RawFrame, RenderOptions};
use raw_pipeline::source::RenderedSource;
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
    timestamps: bool,
    active: Arc<RwLock<ActiveRenderer>>,
    label: Arc<RwLock<Option<String>>>,
    software_gpu: Arc<RwLock<bool>>,
    last_rebuild: Arc<RwLock<Option<Instant>>>,
}

impl RenderDevice {
    pub fn new(mode: RendererMode, texture_cache_bytes: u64, timestamps: bool) -> Self {
        let init = init_gpu(mode, gpu_options(texture_cache_bytes, timestamps));
        Self {
            gpu: Arc::new(RwLock::new(init.renderer)),
            cpu: Arc::new(CpuRenderer::new()),
            mode,
            texture_cache_bytes,
            timestamps,
            active: Arc::new(RwLock::new(init.active)),
            label: Arc::new(RwLock::new(init.label)),
            software_gpu: Arc::new(RwLock::new(init.software)),
            last_rebuild: Arc::new(RwLock::new(None)),
        }
    }

    pub fn active(&self) -> ActiveRenderer {
        *self.active.read().unwrap()
    }

    pub fn gpu_timestamps(&self) -> bool {
        self.gpu
            .read()
            .unwrap()
            .as_ref()
            .is_some_and(|g| g.gpu_timestamps())
    }

    pub fn label(&self) -> Option<String> {
        self.label.read().unwrap().clone()
    }

    pub fn software_gpu(&self) -> bool {
        *self.software_gpu.read().unwrap()
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
            orient = ?frame.meta.orientation,
            sensor_w = frame.meta.width,
            sensor_h = frame.meta.height,
            "render orientation"
        );
        self.run(
            |gpu| gpu.render_with_cancel(frame, edits, opts, cancel),
            |cpu| cpu.render_with_cancel(frame, edits, opts, cancel),
        )
    }

    pub fn source_blocking(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        opts: &RenderOptions,
        cancel: Option<&CancelToken>,
    ) -> Result<RenderedSource, PipelineError> {
        self.run(
            |gpu| gpu.render_source(frame, edits, opts, cancel),
            |cpu| cpu.render_source(frame, edits, opts, cancel),
        )
    }

    fn run<T>(
        &self,
        on_gpu: impl FnOnce(&GpuRenderer) -> Result<T, PipelineError>,
        on_cpu: impl FnOnce(&CpuRenderer) -> Result<T, PipelineError>,
    ) -> Result<T, PipelineError> {
        if matches!(self.mode, RendererMode::Cpu) {
            return on_cpu(&self.cpu);
        }
        let gpu = self.gpu_or_rebuild();
        if let Some(g) = gpu {
            if g.is_lost() {
                self.handle_device_lost();
            } else {
                match on_gpu(&g) {
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
        on_cpu(&self.cpu)
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
        match GpuRenderer::with_options(gpu_options(self.texture_cache_bytes, self.timestamps)) {
            Ok(r) => {
                let label = r.adapter_label();
                tracing::info!(adapter = %label, "gpu renderer rebuilt after device loss");
                let software = r.is_software_adapter();
                let arc = Arc::new(r);
                *self.gpu.write().unwrap() = Some(arc.clone());
                *self.label.write().unwrap() = Some(label);
                *self.software_gpu.write().unwrap() = software;
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
        *self.software_gpu.write().unwrap() = false;
        *self.active.write().unwrap() = ActiveRenderer::Cpu;
        *self.last_rebuild.write().unwrap() = Some(Instant::now());
    }
}

struct GpuInit {
    renderer: Option<Arc<GpuRenderer>>,
    active: ActiveRenderer,
    label: Option<String>,
    software: bool,
}

fn gpu_options(texture_cache_max_bytes: u64, timestamps: bool) -> GpuRendererOptions {
    GpuRendererOptions {
        texture_cache_max_bytes,
        timestamps,
    }
}

fn init_gpu(mode: RendererMode, options: GpuRendererOptions) -> GpuInit {
    let cpu_only = GpuInit {
        renderer: None,
        active: ActiveRenderer::Cpu,
        label: None,
        software: false,
    };
    if matches!(mode, RendererMode::Cpu) {
        return cpu_only;
    }
    match GpuRenderer::with_options(options) {
        Ok(r) => {
            let label = r.adapter_label();
            let software = r.is_software_adapter();
            if software {
                tracing::info!(
                    adapter = %label,
                    "no hardware gpu found; using a software vulkan rasterizer, which still beats the cpu renderer"
                );
            } else {
                tracing::info!(adapter = %label, "gpu renderer initialized");
            }
            GpuInit {
                renderer: Some(Arc::new(r)),
                active: ActiveRenderer::Gpu,
                label: Some(label),
                software,
            }
        }
        Err(e) => {
            if matches!(mode, RendererMode::Gpu) {
                tracing::error!(error = %e, "gpu requested but unavailable; falling back to cpu");
            } else {
                tracing::warn!(error = %e, "gpu unavailable; using cpu");
            }
            cpu_only
        }
    }
}
