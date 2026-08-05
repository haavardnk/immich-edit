use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use bytes::Bytes;
use raw_pipeline::CancelToken;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{RawFrame, RenderOptions};
use raw_pipeline::mask_raster::{MaskRaster, RasterMap};
use raw_pipeline::{GpuRenderer, GpuRendererOptions, PipelineError, RenderedImage};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::RendererMode;
use crate::immich::{ImmichClient, ImmichError};
use crate::services::raster_store::RasterStore;
use crate::services::raw_frame_cache::{FrameCacheKey, RawFrameCache};
use crate::services::render_telemetry::{RenderTelemetry, RendererKind};

const GPU_REBUILD_MIN_INTERVAL: Duration = Duration::from_secs(30);
const MB: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub struct RenderCacheOptions {
    pub raw_frame_cache_mb: u64,
    pub quality_frame_cache_mb: u64,
    pub gpu_texture_cache_mb: u64,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct FrameCacheBytes {
    pub preview_used: u64,
    pub preview_cap: u64,
    pub quality_used: u64,
    pub quality_cap: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderIdentity {
    pub owner: Uuid,
    pub server_epoch: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("upstream: {0}")]
    Upstream(#[from] ImmichError),
    #[error("pipeline: {0}")]
    Pipeline(#[from] PipelineError),
    #[error("lut: {0}")]
    Lut(String),
    #[error("dcp: {0}")]
    Dcp(String),
}

#[derive(Clone)]
pub struct RenderService {
    frames: Arc<Mutex<RawFrameCache>>,
    quality_frames: Arc<Mutex<RawFrameCache>>,
    gpu: Arc<RwLock<Option<Arc<GpuRenderer>>>>,
    gpu_mode: RendererMode,
    gpu_texture_cache_bytes: u64,
    active: Arc<RwLock<ActiveRenderer>>,
    gpu_label: Arc<RwLock<Option<String>>>,
    last_rebuild: Arc<RwLock<Option<Instant>>>,
    rasters: RasterStore,
    luts: crate::services::lut_store::LutStore,
    dcp: crate::services::dcp_store::DcpStore,
    telemetry: RenderTelemetry,
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
    pub async fn dcp_revision(&self) -> Result<String, RenderError> {
        self.dcp
            .revision()
            .await
            .map_err(|e| RenderError::Dcp(e.to_string()))
    }

    pub fn new(
        cache: RenderCacheOptions,
        mode: RendererMode,
        rasters: RasterStore,
        luts: crate::services::lut_store::LutStore,
        dcp: crate::services::dcp_store::DcpStore,
    ) -> Self {
        let gpu_texture_cache_bytes = cache.gpu_texture_cache_mb.saturating_mul(MB);
        let (gpu, active, gpu_label) = init_gpu(mode, gpu_texture_cache_bytes);
        Self {
            frames: Arc::new(Mutex::new(RawFrameCache::new(
                cache.raw_frame_cache_mb.saturating_mul(MB),
            ))),
            quality_frames: Arc::new(Mutex::new(RawFrameCache::new(
                cache.quality_frame_cache_mb.saturating_mul(MB),
            ))),
            gpu: Arc::new(RwLock::new(gpu)),
            gpu_mode: mode,
            gpu_texture_cache_bytes,
            active: Arc::new(RwLock::new(active)),
            gpu_label: Arc::new(RwLock::new(gpu_label)),
            last_rebuild: Arc::new(RwLock::new(None)),
            rasters,
            luts,
            dcp,
            telemetry: RenderTelemetry::new(),
        }
    }

    pub fn active(&self) -> ActiveRenderer {
        *self.active.read().unwrap()
    }

    pub fn gpu_label(&self) -> Option<String> {
        self.gpu_label.read().unwrap().clone()
    }

    pub fn telemetry(&self) -> &RenderTelemetry {
        &self.telemetry
    }

    pub fn gpu_pool_stats(&self) -> Option<raw_pipeline::GpuPoolStats> {
        self.gpu.read().unwrap().as_ref().map(|g| g.pool_stats())
    }

    pub async fn frame_cache_bytes(&self) -> FrameCacheBytes {
        let preview = self.frames.lock().await;
        let quality = self.quality_frames.lock().await;
        FrameCacheBytes {
            preview_used: preview.current_bytes(),
            preview_cap: preview.max_bytes(),
            quality_used: quality.current_bytes(),
            quality_cap: quality.max_bytes(),
        }
    }

    pub async fn clear_frame_caches(&self) {
        self.frames.lock().await.clear();
        self.quality_frames.lock().await.clear();
    }

    pub async fn frame(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        asset_id: Uuid,
    ) -> Result<Arc<RawFrame>, RenderError> {
        let key = FrameCacheKey {
            server_epoch: identity.server_epoch,
            asset_id,
        };
        if let Some(f) = self.frames.lock().await.get(&key) {
            return Ok(f);
        }
        let bytes = immich.original(asset_id).await?;
        let frame = decode_blocking(bytes).await?;
        let frame = Arc::new(frame);
        self.frames.lock().await.put(key, frame.clone());
        Ok(frame)
    }

    pub async fn quality_frame(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        asset_id: Uuid,
    ) -> Result<Arc<RawFrame>, RenderError> {
        let key = FrameCacheKey {
            server_epoch: identity.server_epoch,
            asset_id,
        };
        if let Some(f) = self.quality_frames.lock().await.get(&key) {
            return Ok(f);
        }
        let bytes = immich.original(asset_id).await?;
        let frame = decode_quality_blocking(bytes).await?;
        self.quality_frames.lock().await.put(key, frame.clone());
        Ok(frame)
    }

    pub async fn render(
        &self,
        identity: RenderIdentity,
        immich: ImmichClient,
        asset_id: Uuid,
        edits: Edits,
        mut options: RenderOptions,
        cancel: Option<CancelToken>,
    ) -> Result<RenderedImage, RenderError> {
        let frame = if options.quality {
            self.quality_frame(identity, &immich, asset_id).await?
        } else {
            self.frame(identity, &immich, asset_id).await?
        };
        options.rasters = self.load_rasters_for(identity, &edits).await;
        options.luts = self.load_luts_for(&edits).await?;
        options.dcp = self.load_dcp_for(&edits, &frame).await?;
        let svc = self.clone();
        let start = Instant::now();
        let result = tokio::task::spawn_blocking(move || {
            svc.render_blocking(&frame, &edits, &options, cancel.as_ref())
        })
        .await
        .map_err(|e| RenderError::Pipeline(PipelineError::Render(format!("join: {e}"))))??;
        let kind = match self.active() {
            ActiveRenderer::Cpu => RendererKind::Cpu,
            ActiveRenderer::Gpu => RendererKind::Gpu,
        };
        self.telemetry.record(kind, start.elapsed());
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
        if matches!(self.gpu_mode, RendererMode::Cpu) {
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
        match GpuRenderer::with_options(GpuRendererOptions {
            texture_pool_max_bytes: self.gpu_texture_cache_bytes,
        }) {
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

    async fn load_rasters_for(&self, identity: RenderIdentity, edits: &Edits) -> RasterMap {
        let ids = edits.referenced_raster_ids();
        let mut map: RasterMap = RasterMap::with_capacity(ids.len());
        for id in ids {
            match self
                .rasters
                .load(identity.server_epoch, identity.owner, &id)
                .await
            {
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

    async fn load_luts_for(&self, edits: &Edits) -> Result<raw_pipeline::lut::LutMap, RenderError> {
        let mut map = raw_pipeline::lut::empty_luts();
        if let Some(id) = edits.referenced_lut_id() {
            let lut = self
                .luts
                .load(&id)
                .await
                .map_err(|e| RenderError::Lut(format!("{id}: {e}")))?;
            map.insert(id, lut);
        }
        Ok(map)
    }

    async fn load_dcp_for(
        &self,
        edits: &Edits,
        frame: &RawFrame,
    ) -> Result<Option<Arc<raw_pipeline::dcp::DcpProfile>>, RenderError> {
        use raw_pipeline::edits::DcpMode;
        let dcp = &edits.color.dcp;
        if !frame.is_raw || !dcp.is_active() {
            return Ok(None);
        }
        match dcp.mode {
            DcpMode::Off => Ok(None),
            DcpMode::Profile => match dcp.referenced_profile_id() {
                Some(id) => {
                    let p = self
                        .dcp
                        .load(&id)
                        .await
                        .map_err(|e| RenderError::Dcp(format!("{id}: {e}")))?;
                    Ok(Some(p))
                }
                None => Ok(None),
            },
            DcpMode::Auto => self
                .dcp
                .match_camera(&frame.model)
                .await
                .map_err(|e| RenderError::Dcp(e.to_string())),
        }
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
