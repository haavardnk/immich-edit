use std::sync::Arc;
use std::time::Instant;

use bytes::Bytes;
use raw_pipeline::CancelToken;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{RawFrame, RenderOptions};
use raw_pipeline::{PipelineError, RenderedImage};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::RendererMode;
use crate::immich::{ImmichClient, ImmichError};
use crate::services::raster_store::RasterStore;
use crate::services::raw_frame_cache::{FrameCacheKey, RawFrameCache};
use crate::services::render_telemetry::{RenderTelemetry, RendererKind};

mod device;
mod inputs;

pub use device::ActiveRenderer;

use device::RenderDevice;
use inputs::RenderInputs;

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
    device: RenderDevice,
    inputs: RenderInputs,
    telemetry: RenderTelemetry,
}

impl RenderService {
    pub async fn dcp_revision(&self) -> Result<String, RenderError> {
        self.inputs.dcp_revision().await
    }

    pub fn new(
        cache: RenderCacheOptions,
        mode: RendererMode,
        rasters: RasterStore,
        luts: crate::services::lut_store::LutStore,
        dcp: crate::services::dcp_store::DcpStore,
    ) -> Self {
        Self {
            frames: Arc::new(Mutex::new(RawFrameCache::new(
                cache.raw_frame_cache_mb.saturating_mul(MB),
            ))),
            quality_frames: Arc::new(Mutex::new(RawFrameCache::new(
                cache.quality_frame_cache_mb.saturating_mul(MB),
            ))),
            device: RenderDevice::new(mode, cache.gpu_texture_cache_mb.saturating_mul(MB)),
            inputs: RenderInputs::new(rasters, luts, dcp),
            telemetry: RenderTelemetry::new(),
        }
    }

    pub fn active(&self) -> ActiveRenderer {
        self.device.active()
    }

    pub fn gpu_label(&self) -> Option<String> {
        self.device.label()
    }

    pub fn software_gpu(&self) -> bool {
        self.device.software_gpu()
    }

    pub fn telemetry(&self) -> &RenderTelemetry {
        &self.telemetry
    }

    pub fn gpu_pool_stats(&self) -> Option<raw_pipeline::GpuPoolStats> {
        self.device.pool_stats()
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

    async fn cached_frame<F, Fut>(
        cache: &Mutex<RawFrameCache>,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
        decode: F,
    ) -> Result<Arc<RawFrame>, RenderError>
    where
        F: FnOnce(Bytes) -> Fut,
        Fut: Future<Output = Result<Arc<RawFrame>, PipelineError>>,
    {
        let key = FrameCacheKey {
            server_epoch: identity.server_epoch,
            owner: identity.owner,
            asset_id: source,
        };
        if let Some(f) = cache.lock().await.get(&key) {
            return Ok(f);
        }
        let bytes = immich.original(source).await?;
        let frame = decode(bytes).await?;
        cache.lock().await.put(key, frame.clone());
        Ok(frame)
    }

    pub async fn frame(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
    ) -> Result<Arc<RawFrame>, RenderError> {
        Self::cached_frame(&self.frames, identity, immich, source, decode_blocking).await
    }

    pub async fn quality_frame(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
    ) -> Result<Arc<RawFrame>, RenderError> {
        Self::cached_frame(
            &self.quality_frames,
            identity,
            immich,
            source,
            decode_quality_blocking,
        )
        .await
    }

    pub async fn render(
        &self,
        identity: RenderIdentity,
        immich: ImmichClient,
        source: Uuid,
        edits: Edits,
        mut options: RenderOptions,
        cancel: Option<CancelToken>,
    ) -> Result<RenderedImage, RenderError> {
        let frame = if options.quality {
            self.quality_frame(identity, &immich, source).await?
        } else {
            self.frame(identity, &immich, source).await?
        };
        let mut edits = edits;
        if frame.is_raw {
            edits.lens = self.inputs.resolve_lens(&immich, source, edits.lens).await;
        }
        options.rasters = self.inputs.rasters_for(identity, &edits).await;
        options.luts = self.inputs.luts_for(&edits).await?;
        options.dcp = self.inputs.dcp_for(&edits, &frame).await?;
        let device = self.device.clone();
        let start = Instant::now();
        let result = tokio::task::spawn_blocking(move || {
            device.render_blocking(&frame, &edits, &options, cancel.as_ref())
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
}

async fn decode_blocking(bytes: Bytes) -> Result<Arc<RawFrame>, PipelineError> {
    let frame = tokio::task::spawn_blocking(move || raw_pipeline::decode::decode(&bytes))
        .await
        .map_err(|e| PipelineError::Decode(format!("join: {e}")))?;
    Ok(Arc::new(frame?))
}

async fn decode_quality_blocking(bytes: Bytes) -> Result<Arc<RawFrame>, PipelineError> {
    let frame = tokio::task::spawn_blocking(move || raw_pipeline::decode::decode_quality(&bytes))
        .await
        .map_err(|e| PipelineError::Decode(format!("join: {e}")))?;
    Ok(Arc::new(frame?))
}
