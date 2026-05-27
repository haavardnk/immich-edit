use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use bytes::Bytes;
use raw_pipeline::CancelToken;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{RawFrame, RenderOptions};
use raw_pipeline::mask_raster::{MaskRaster, RasterMap};
use raw_pipeline::{GpuRenderer, PipelineError, RenderedImage};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::RendererMode;
use crate::immich::{ImmichClient, ImmichError};
use crate::services::raster_store::RasterStore;

const GPU_REBUILD_MIN_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("upstream: {0}")]
    Upstream(#[from] ImmichError),
    #[error("pipeline: {0}")]
    Pipeline(#[from] PipelineError),
}

#[derive(Clone)]
pub struct RenderService {
    immich: ImmichClient,
    frames: Arc<Mutex<lru::LruCache<Uuid, Arc<RawFrame>>>>,
    gpu: Arc<RwLock<Option<Arc<GpuRenderer>>>>,
    gpu_mode: RendererMode,
    active: Arc<RwLock<ActiveRenderer>>,
    gpu_label: Arc<RwLock<Option<String>>>,
    last_rebuild: Arc<RwLock<Option<Instant>>>,
    rasters: RasterStore,
}

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

impl RenderService {
    pub fn new(
        immich: ImmichClient,
        max_items: usize,
        mode: RendererMode,
        rasters: RasterStore,
    ) -> Self {
        let cap = std::num::NonZeroUsize::new(max_items.max(1)).unwrap();
        let (gpu, active, gpu_label) = init_gpu(mode);
        Self {
            immich,
            frames: Arc::new(Mutex::new(lru::LruCache::new(cap))),
            gpu: Arc::new(RwLock::new(gpu)),
            gpu_mode: mode,
            active: Arc::new(RwLock::new(active)),
            gpu_label: Arc::new(RwLock::new(gpu_label)),
            last_rebuild: Arc::new(RwLock::new(None)),
            rasters,
        }
    }

    pub fn active(&self) -> ActiveRenderer {
        *self.active.read().unwrap()
    }

    pub fn gpu_label(&self) -> Option<String> {
        self.gpu_label.read().unwrap().clone()
    }

    pub async fn frame(&self, asset_id: Uuid) -> Result<Arc<RawFrame>, RenderError> {
        if let Some(f) = self.frames.lock().await.get(&asset_id).cloned() {
            return Ok(f);
        }
        let bytes = self.immich.original(asset_id).await?;
        let frame = decode_blocking(bytes).await?;
        let frame = Arc::new(frame);
        self.frames.lock().await.put(asset_id, frame.clone());
        Ok(frame)
    }

    pub async fn quality_frame(&self, asset_id: Uuid) -> Result<Arc<RawFrame>, RenderError> {
        let bytes = self.immich.original(asset_id).await?;
        Ok(decode_quality_blocking(bytes).await?)
    }

    pub async fn render(
        &self,
        asset_id: Uuid,
        edits: Edits,
        mut options: RenderOptions,
        cancel: Option<CancelToken>,
    ) -> Result<RenderedImage, RenderError> {
        let frame = if options.quality {
            let bytes = self.immich.original(asset_id).await?;
            decode_quality_blocking(bytes).await?
        } else {
            self.frame(asset_id).await?
        };
        options.rasters = self.load_rasters_for(&edits).await;
        let svc = self.clone();
        let result = tokio::task::spawn_blocking(move || {
            svc.render_blocking(&frame, &edits, &options, cancel.as_ref())
        })
        .await
        .map_err(|e| RenderError::Pipeline(PipelineError::Render(format!("join: {e}"))))??;
        Ok(result)
    }

    fn render_blocking(
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
        let mask_preview = matches!(
            opts.preview_mode,
            raw_pipeline::frame::PreviewMode::MaskWeight { .. }
        );
        if matches!(self.gpu_mode, RendererMode::Cpu) || mask_preview {
            return raw_pipeline::cpu::render_with_cancel(frame, edits, opts, cancel);
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
        raw_pipeline::cpu::render_with_cancel(frame, edits, opts, cancel)
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
        match GpuRenderer::new() {
            Ok(r) => {
                let label = r.adapter_label();
                tracing::info!(adapter = %label, "gpu renderer rebuilt after device loss");
                let arc = Arc::new(r);
                *self.gpu.write().unwrap() = Some(arc.clone());
                *self.gpu_label.write().unwrap() = Some(label);
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
        *self.gpu_label.write().unwrap() = None;
        *self.active.write().unwrap() = ActiveRenderer::Cpu;
        *self.last_rebuild.write().unwrap() = Some(Instant::now());
    }

    async fn load_rasters_for(&self, edits: &Edits) -> RasterMap {
        let ids = edits.referenced_raster_ids();
        let mut map: RasterMap = RasterMap::with_capacity(ids.len());
        for id in ids {
            match self.rasters.load(&id).await {
                Ok((meta, bytes)) => {
                    if let Some(r) = MaskRaster::new(meta.width, meta.height, bytes) {
                        map.insert(id, Arc::new(r));
                    }
                }
                Err(e) => {
                    tracing::warn!(raster_id = %id, error = %e, "raster load failed");
                }
            }
        }
        map
    }
}

fn init_gpu(mode: RendererMode) -> (Option<Arc<GpuRenderer>>, ActiveRenderer, Option<String>) {
    if matches!(mode, RendererMode::Cpu) {
        return (None, ActiveRenderer::Cpu, None);
    }
    match GpuRenderer::new() {
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

async fn decode_blocking(bytes: Bytes) -> Result<RawFrame, PipelineError> {
    tokio::task::spawn_blocking(move || raw_pipeline::decode::decode(&bytes))
        .await
        .map_err(|e| PipelineError::Decode(format!("join: {e}")))?
}

async fn decode_quality_blocking(bytes: Bytes) -> Result<Arc<RawFrame>, PipelineError> {
    let frame = tokio::task::spawn_blocking(move || raw_pipeline::decode::decode_quality(&bytes))
        .await
        .map_err(|e| PipelineError::Decode(format!("join: {e}")))?;
    Ok(Arc::new(frame?))
}
