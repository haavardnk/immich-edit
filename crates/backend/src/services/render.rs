use std::sync::Arc;

use bytes::Bytes;
use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{RawFrame, RenderOptions};
use raw_pipeline::{CpuRenderer, PipelineError, RenderedImage, Renderer};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::immich::{ImmichClient, ImmichError};

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
}

impl RenderService {
    pub fn new(immich: ImmichClient, max_items: usize) -> Self {
        let cap = std::num::NonZeroUsize::new(max_items.max(1)).unwrap();
        Self {
            immich,
            frames: Arc::new(Mutex::new(lru::LruCache::new(cap))),
        }
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

    pub async fn render(
        &self,
        asset_id: Uuid,
        edits: Edits,
        max_edge: u32,
    ) -> Result<RenderedImage, RenderError> {
        let frame = self.frame(asset_id).await?;
        let opts = RenderOptions { max_edge };
        let result = tokio::task::spawn_blocking(move || {
            let renderer = CpuRenderer;
            renderer.render(&frame, &edits, &opts)
        })
        .await
        .map_err(|e| RenderError::Pipeline(PipelineError::Render(format!("join: {e}"))))??;
        Ok(result)
    }
}

async fn decode_blocking(bytes: Bytes) -> Result<RawFrame, PipelineError> {
    tokio::task::spawn_blocking(move || raw_pipeline::decode::decode(&bytes))
        .await
        .map_err(|e| PipelineError::Decode(format!("join: {e}")))?
}
