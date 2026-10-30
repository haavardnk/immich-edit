use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::Bytes;
use raw_pipeline::CancelToken;
use raw_pipeline::edits::{Edits, LensEdits};
use raw_pipeline::frame::{RawFrame, RenderOptions};
use raw_pipeline::gpu::memory::GpuMemory;
use raw_pipeline::source::SourceCoding;
use raw_pipeline::timing::StageTiming;
use raw_pipeline::{GpuRendererOptions, PipelineError, RenderedImage};
use uuid::Uuid;

use crate::config::RendererMode;
use crate::host::host_info;
use crate::immich::{ImmichClient, ImmichError};
use crate::services::raster_store::RasterStore;
use crate::services::render_telemetry::RenderTelemetry;

mod denoise;
mod device;
mod frames;
mod inputs;
mod lens;
mod warm;

pub use denoise::{AssetRevision, DenoiseInputs, OwnerPlanes, PlaneState};
pub use device::RendererKind;
pub use warm::WARM_NEIGHBOURS;

use device::RenderDevice;
use frames::FrameStore;
use inputs::RenderInputs;
use lens::LensProfiles;
use warm::{FrameWarmer, WarmJob};

const MB: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub struct RenderCacheOptions {
    pub raw_frame_cache_mb: u64,
    pub quality_frame_cache_mb: u64,
    pub gpu_texture_cache_mb: Option<u64>,
    pub gpu_memory_budget_mb: Option<u64>,
    pub gpu_tile_edge: Option<u32>,
}

impl RenderCacheOptions {
    fn gpu(&self, timestamps: bool) -> GpuRendererOptions {
        let frame_caches =
            (self.raw_frame_cache_mb + self.quality_frame_cache_mb).saturating_mul(MB);
        GpuRendererOptions {
            texture_cache_max_bytes: self.gpu_texture_cache_mb.map(|mb| mb.saturating_mul(MB)),
            memory_budget_bytes: self.gpu_memory_budget_mb.map(|mb| mb.saturating_mul(MB)),
            host_budget_bytes: host_info()
                .memory_bytes()
                .map(|bytes| bytes.saturating_sub(frame_caches)),
            tile_edge: self.gpu_tile_edge,
            display_tile_edge: None,
            timestamps,
        }
    }
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
    warmer: FrameWarmer,
    device: RenderDevice,
    inputs: RenderInputs,
    denoise: DenoiseInputs,
    lens: LensProfiles,
    telemetry: RenderTelemetry,
}

impl RenderService {
    pub async fn render_revision(&self) -> Result<String, RenderError> {
        let dcp = self.inputs.dcp_revision().await?;
        Ok(format!(
            "{dcp}-l{}-v{}",
            u8::from(self.lens.auto()),
            env!("CARGO_PKG_VERSION")
        ))
    }

    pub async fn asset_revision(
        &self,
        identity: RenderIdentity,
        source: Uuid,
        edits: &Edits,
    ) -> Result<AssetRevision, RenderError> {
        let base = self.render_revision().await?;
        let plane = self.denoise.state(identity, source, edits).await;
        Ok(AssetRevision::new(base, plane))
    }

    pub fn denoise(&self) -> &DenoiseInputs {
        &self.denoise
    }

    pub fn lens_auto(&self) -> bool {
        self.lens.auto()
    }

    pub fn set_lens_auto(&self, enabled: bool) {
        self.lens.set_auto(enabled);
    }

    pub fn new(
        cache: RenderCacheOptions,
        mode: RendererMode,
        gpu_timestamps: bool,
        rasters: RasterStore,
        luts: crate::services::lut_store::LutStore,
        dcp: crate::services::dcp_store::DcpStore,
        denoise: DenoiseInputs,
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
            warmer: FrameWarmer::default(),
            device: RenderDevice::new(mode, cache.gpu(gpu_timestamps)),
            inputs: RenderInputs::new(rasters, luts, dcp),
            denoise,
            lens: LensProfiles::default(),
            telemetry,
        }
    }

    pub fn active(&self) -> RendererKind {
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

    pub fn gpu_memory(&self) -> Option<GpuMemory> {
        self.device.memory()
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

    pub async fn dcp_profile(
        &self,
        edits: &Edits,
        frame: &RawFrame,
    ) -> Result<Option<Arc<raw_pipeline::dcp::DcpProfile>>, RenderError> {
        Ok(self
            .inputs
            .dcp_for(edits, frame)
            .await?
            .map(|selection| selection.profile))
    }

    pub async fn warm(
        &self,
        identity: RenderIdentity,
        immich: ImmichClient,
        sources: Vec<Uuid>,
    ) -> bool {
        let job = WarmJob {
            identity,
            immich,
            sources,
        };
        self.warmer.submit(&self.frames, job).await
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
        options: RenderOptions,
        cancel: Option<CancelToken>,
    ) -> Result<RenderedImage, RenderError> {
        self.render_timed(identity, immich, source, edits, options, cancel)
            .await
            .map(|timed| timed.image)
    }

    async fn auto_lens(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
        frame: &RawFrame,
        lens: LensEdits,
    ) -> LensEdits {
        if !frame.meta.is_raw || frame.embedded_lens.is_corrected() {
            return lens;
        }
        self.lens.resolve(identity, immich, source, lens).await
    }

    pub async fn render_timed(
        &self,
        identity: RenderIdentity,
        immich: ImmichClient,
        source: Uuid,
        edits: Edits,
        mut options: RenderOptions,
        cancel: Option<CancelToken>,
    ) -> Result<TimedRender, RenderError> {
        let frame = if options.quality {
            self.quality_frame(identity, &immich, source).await?
        } else {
            self.frame(identity, &immich, source).await?
        };
        let mut edits = edits;
        edits.lens = self
            .auto_lens(identity, &immich, source, &frame, edits.lens)
            .await;
        if options.denoise.is_none() {
            options.denoise = self.denoise.plane(identity, source, &edits, &frame).await;
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
        let wall = start.elapsed();
        let kind = RendererKind::from_label(&result.renderer);
        self.telemetry
            .record(kind, wall, &result.timings, result.tiling);
        Ok(TimedRender {
            image: result,
            wall,
        })
    }

    pub async fn source(
        &self,
        identity: RenderIdentity,
        immich: ImmichClient,
        source: Uuid,
        request: SourceRequest,
        cancel: Option<CancelToken>,
    ) -> Result<EncodedSource, RenderError> {
        let SourceRequest {
            mut edits,
            mut options,
            coding,
        } = request;
        let frame = self.frame(identity, &immich, source).await?;
        edits.lens = self
            .auto_lens(identity, &immich, source, &frame, edits.lens)
            .await;
        options.denoise = self.denoise.plane(identity, source, &edits, &frame).await;
        if options.roi.is_some() {
            options.rasters = self.inputs.rasters_for(identity, &edits).await;
        }
        let dcp = self.inputs.dcp_for(&edits, &frame).await?;
        let dcp_id = dcp.as_ref().map(|selection| selection.id.clone());
        options.dcp = dcp.map(|selection| selection.profile);
        let device = self.device.clone();
        let start = Instant::now();
        let (bytes, renderer, timings, tiling) = tokio::task::spawn_blocking(move || {
            let rendered = device.source_blocking(&frame, &edits, &options, cancel.as_ref())?;
            let encode_start = Instant::now();
            let bytes = raw_pipeline::source::encode(&rendered.image, coding)?;
            let mut timings = rendered.timings;
            timings.push(StageTiming {
                stage: raw_pipeline::timing::ENCODE,
                wall: encode_start.elapsed(),
                gpu: None,
            });
            Ok::<_, PipelineError>((bytes, rendered.renderer, timings, rendered.tiling))
        })
        .await
        .map_err(|e| RenderError::Pipeline(PipelineError::Render(format!("join: {e}"))))??;
        let kind = RendererKind::from_label(&renderer);
        self.telemetry
            .record(kind, start.elapsed(), &timings, tiling);
        Ok(EncodedSource { bytes, dcp_id })
    }
}

pub struct TimedRender {
    pub image: RenderedImage,
    pub wall: Duration,
}

pub struct SourceRequest {
    pub edits: Edits,
    pub options: RenderOptions,
    pub coding: SourceCoding,
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
