use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use raw_pipeline::histogram::Histogram;
use serde::Serialize;
use tokio::sync::Mutex;
use uuid::Uuid;

const DEFAULT_CAP: usize = 512;

#[derive(Debug, Clone, Serialize)]
pub struct PreviewMeta {
    pub asset_id: Uuid,
    pub width: u32,
    pub height: u32,
    pub source_w: u32,
    pub source_h: u32,
    pub renderer: String,
    pub histogram: Histogram,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linear_histogram: Option<Histogram>,
}

#[derive(Clone)]
pub struct PreviewMetaStore {
    inner: Arc<Mutex<LruCache<Uuid, PreviewMeta>>>,
}

impl Default for PreviewMetaStore {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_CAP)
    }
}

impl PreviewMetaStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(cap: usize) -> Self {
        let n = NonZeroUsize::new(cap.max(1)).unwrap();
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(n))),
        }
    }

    pub async fn put(&self, meta: PreviewMeta) -> Uuid {
        let id = Uuid::new_v4();
        self.inner.lock().await.put(id, meta);
        id
    }

    pub async fn get(&self, id: Uuid) -> Option<PreviewMeta> {
        self.inner.lock().await.get(&id).cloned()
    }
}
