use std::sync::Arc;
use std::time::Instant;

use bytes::Bytes;
use raw_pipeline::PipelineError;
use raw_pipeline::decode::Develop;
use raw_pipeline::frame::RawFrame;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::immich::ImmichClient;
use crate::services::raw_frame_cache::{FrameCacheKey, RawFrameCache};
use crate::services::render_telemetry::RenderTelemetry;
use crate::services::single_flight::SingleFlight;

use super::{RenderError, RenderIdentity};

type FrameResult = Result<Arc<RawFrame>, RenderError>;

#[derive(Clone, Copy)]
pub struct FrameDecoder {
    develop: Develop,
    budget: Option<u64>,
}

impl FrameDecoder {
    pub fn new(develop: Develop, budget: Option<u64>) -> Self {
        Self { develop, budget }
    }

    async fn decode(self, bytes: Bytes) -> Result<Arc<RawFrame>, PipelineError> {
        let frame = tokio::task::spawn_blocking(move || {
            raw_pipeline::decode::decode_within(&bytes, self.develop, self.budget)
        })
        .await
        .map_err(|e| PipelineError::Decode(format!("join: {e}")))?;
        Ok(Arc::new(frame?))
    }
}

#[derive(Clone)]
pub struct FrameStore {
    cache: Arc<Mutex<RawFrameCache>>,
    inflight: SingleFlight<FrameCacheKey, FrameResult>,
    decoder: FrameDecoder,
    telemetry: RenderTelemetry,
}

impl FrameStore {
    pub fn new(max_bytes: u64, decoder: FrameDecoder, telemetry: RenderTelemetry) -> Self {
        Self {
            cache: Arc::new(Mutex::new(RawFrameCache::new(max_bytes))),
            inflight: SingleFlight::default(),
            decoder,
            telemetry,
        }
    }

    pub async fn used_bytes(&self) -> u64 {
        self.cache.lock().await.current_bytes()
    }

    pub async fn max_bytes(&self) -> u64 {
        self.cache.lock().await.max_bytes()
    }

    pub async fn holds_frames_like_largest(&self, count: u64) -> bool {
        self.cache.lock().await.holds_frames_like_largest(count)
    }

    pub async fn clear(&self) {
        self.cache.lock().await.clear();
    }

    pub async fn get_or_load(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
    ) -> FrameResult {
        let key = FrameCacheKey {
            server_epoch: identity.server_epoch,
            owner: identity.owner,
            asset_id: source,
        };
        if let Some(frame) = self.cache.lock().await.get(&key) {
            self.telemetry.record_frame_hit();
            return Ok(frame);
        }
        self.telemetry.record_frame_miss();
        let cache = self.cache.clone();
        let immich = immich.clone();
        let telemetry = self.telemetry.clone();
        let decoder = self.decoder;
        let loaded = self
            .inflight
            .run(key, move || async move {
                let result = load(&immich, source, decoder, &telemetry).await;
                if let Ok(frame) = &result {
                    cache.lock().await.put(key, frame.clone());
                }
                result
            })
            .await;
        loaded.unwrap_or_else(|| {
            Err(RenderError::Pipeline(PipelineError::Decode(
                "frame loader stopped".into(),
            )))
        })
    }
}

async fn load(
    immich: &ImmichClient,
    source: Uuid,
    decoder: FrameDecoder,
    telemetry: &RenderTelemetry,
) -> FrameResult {
    let started = Instant::now();
    let bytes = immich.original(source).await?;
    let fetched = started.elapsed();
    let started = Instant::now();
    let frame = decoder.decode(bytes).await?;
    telemetry.record_frame_load(fetched, started.elapsed());
    Ok(frame)
}
