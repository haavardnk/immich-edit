use std::sync::Arc;
use std::time::Instant;

use bytes::Bytes;
use raw_pipeline::CancelToken;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{RawFrame, RenderOptions};
use raw_pipeline::timing::StageTiming;
use raw_pipeline::{PipelineError, RenderedImage};
use uuid::Uuid;

use crate::config::RendererMode;
use crate::immich::{ImmichClient, ImmichError};
use crate::services::raster_store::RasterStore;
use crate::services::render_telemetry::{RenderTelemetry, RendererKind};

mod device;
mod frames;
mod inputs;

pub use device::ActiveRenderer;

use device::RenderDevice;
use frames::FrameStore;
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

#[derive(Debug, Clone, thiserror::Error)]
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
    frames: FrameStore,
    quality_frames: FrameStore,
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
        gpu_timestamps: bool,
        rasters: RasterStore,
        luts: crate::services::lut_store::LutStore,
        dcp: crate::services::dcp_store::DcpStore,
    ) -> Self {
        let telemetry = RenderTelemetry::new();
        Self {
            frames: FrameStore::new(
                cache.raw_frame_cache_mb.saturating_mul(MB),
                telemetry.clone(),
            ),
            quality_frames: FrameStore::new(
                cache.quality_frame_cache_mb.saturating_mul(MB),
                telemetry.clone(),
            ),
            device: RenderDevice::new(
                mode,
                cache.gpu_texture_cache_mb.saturating_mul(MB),
                gpu_timestamps,
            ),
            inputs: RenderInputs::new(rasters, luts, dcp),
            telemetry,
        }
    }

    pub fn active(&self) -> ActiveRenderer {
        self.device.active()
    }

    pub fn gpu_timestamps(&self) -> bool {
        self.device.gpu_timestamps()
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
        FrameCacheBytes {
            preview_used: self.frames.used_bytes().await,
            preview_cap: self.frames.max_bytes().await,
            quality_used: self.quality_frames.used_bytes().await,
            quality_cap: self.quality_frames.max_bytes().await,
        }
    }

    pub async fn clear_frame_caches(&self) {
        self.frames.clear().await;
        self.quality_frames.clear().await;
    }

    pub async fn frame(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
    ) -> Result<Arc<RawFrame>, RenderError> {
        self.frames
            .get_or_load(identity, immich, source, decode_blocking)
            .await
    }

    pub async fn quality_frame(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
    ) -> Result<Arc<RawFrame>, RenderError> {
        self.quality_frames
            .get_or_load(identity, immich, source, decode_quality_blocking)
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
        if frame.meta.is_raw {
            edits.lens = self.inputs.resolve_lens(&immich, source, edits.lens).await;
        }
        options.rasters = self.inputs.rasters_for(identity, &edits).await;
        options.luts = self.inputs.luts_for(&edits).await?;
        options.dcp = self
            .inputs
            .dcp_for(&edits, &frame)
            .await?
            .map(|selection| selection.profile);
        let device = self.device.clone();
        let start = Instant::now();
        let result = tokio::task::spawn_blocking(move || {
            device.render_blocking(&frame, &edits, &options, cancel.as_ref())
        })
        .await
        .map_err(|e| RenderError::Pipeline(PipelineError::Render(format!("join: {e}"))))??;
        self.telemetry.record(
            RendererKind::from_label(&result.renderer),
            start.elapsed(),
            &result.timings,
        );
        Ok(result)
    }

    pub async fn source(
        &self,
        identity: RenderIdentity,
        immich: ImmichClient,
        source: Uuid,
        edits: Edits,
        mut options: RenderOptions,
        cancel: Option<CancelToken>,
    ) -> Result<EncodedSource, RenderError> {
        let frame = self.frame(identity, &immich, source).await?;
        let mut edits = edits;
        if frame.meta.is_raw {
            edits.lens = self.inputs.resolve_lens(&immich, source, edits.lens).await;
        }
        let dcp = self.inputs.dcp_for(&edits, &frame).await?;
        let dcp_id = dcp.as_ref().map(|selection| selection.id.clone());
        options.dcp = dcp.map(|selection| selection.profile);
        let device = self.device.clone();
        let start = Instant::now();
        let (bytes, renderer, timings) = tokio::task::spawn_blocking(move || {
            let rendered = device.source_blocking(&frame, &edits, &options, cancel.as_ref())?;
            let encode_start = Instant::now();
            let bytes = raw_pipeline::source::encode(&rendered.image)?;
            let mut timings = rendered.timings;
            timings.push(StageTiming {
                stage: raw_pipeline::timing::ENCODE,
                wall: encode_start.elapsed(),
                gpu: None,
            });
            Ok::<_, PipelineError>((bytes, rendered.renderer, timings))
        })
        .await
        .map_err(|e| RenderError::Pipeline(PipelineError::Render(format!("join: {e}"))))??;
        self.telemetry.record(
            RendererKind::from_label(&renderer),
            start.elapsed(),
            &timings,
        );
        Ok(EncodedSource { bytes, dcp_id })
    }
}

pub struct EncodedSource {
    pub bytes: Vec<u8>,
    pub dcp_id: Option<String>,
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
