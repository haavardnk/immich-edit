use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as SyncMutex;

use bytes::Bytes;
use raw_pipeline::PipelineError;
use raw_pipeline::frame::RawFrame;
use tokio::sync::{Mutex, watch};
use uuid::Uuid;

use crate::immich::ImmichClient;
use crate::services::raw_frame_cache::{FrameCacheKey, RawFrameCache};

use super::{RenderError, RenderIdentity};

type FrameResult = Result<Arc<RawFrame>, RenderError>;

#[derive(Clone)]
pub struct FrameStore {
    cache: Arc<Mutex<RawFrameCache>>,
    inflight: Arc<SyncMutex<HashMap<FrameCacheKey, watch::Receiver<Option<FrameResult>>>>>,
}

impl FrameStore {
    pub fn new(max_bytes: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(RawFrameCache::new(max_bytes))),
            inflight: Arc::new(SyncMutex::new(HashMap::new())),
        }
    }

    pub async fn used_bytes(&self) -> u64 {
        self.cache.lock().await.current_bytes()
    }

    pub async fn max_bytes(&self) -> u64 {
        self.cache.lock().await.max_bytes()
    }

    pub async fn clear(&self) {
        self.cache.lock().await.clear();
    }

    pub async fn get_or_load<F, Fut>(
        &self,
        identity: RenderIdentity,
        immich: &ImmichClient,
        source: Uuid,
        decode: F,
    ) -> FrameResult
    where
        F: FnOnce(Bytes) -> Fut + Send + 'static,
        Fut: Future<Output = Result<Arc<RawFrame>, PipelineError>> + Send,
    {
        let key = FrameCacheKey {
            server_epoch: identity.server_epoch,
            owner: identity.owner,
            asset_id: source,
        };
        if let Some(frame) = self.cache.lock().await.get(&key) {
            return Ok(frame);
        }
        let mut rx = self.leader_or_follower(key, immich, source, decode);
        loop {
            let current = rx.borrow_and_update().clone();
            if let Some(result) = current {
                return result;
            }
            if rx.changed().await.is_err() {
                return Err(RenderError::Pipeline(PipelineError::Decode(
                    "frame loader stopped".into(),
                )));
            }
        }
    }

    fn leader_or_follower<F, Fut>(
        &self,
        key: FrameCacheKey,
        immich: &ImmichClient,
        source: Uuid,
        decode: F,
    ) -> watch::Receiver<Option<FrameResult>>
    where
        F: FnOnce(Bytes) -> Fut + Send + 'static,
        Fut: Future<Output = Result<Arc<RawFrame>, PipelineError>> + Send,
    {
        let mut inflight = self.inflight.lock().unwrap();
        if let Some(rx) = inflight.get(&key) {
            return rx.clone();
        }
        let (tx, rx) = watch::channel(None);
        inflight.insert(key, rx.clone());
        let cache = self.cache.clone();
        let pending = self.inflight.clone();
        let immich = immich.clone();
        tokio::spawn(async move {
            let result = load(&immich, source, decode).await;
            if let Ok(frame) = &result {
                cache.lock().await.put(key, frame.clone());
            }
            pending.lock().unwrap().remove(&key);
            let _ = tx.send(Some(result));
        });
        rx
    }
}

async fn load<F, Fut>(immich: &ImmichClient, source: Uuid, decode: F) -> FrameResult
where
    F: FnOnce(Bytes) -> Fut,
    Fut: Future<Output = Result<Arc<RawFrame>, PipelineError>>,
{
    let bytes = immich.original(source).await?;
    Ok(decode(bytes).await?)
}
