use std::collections::HashMap;
use std::sync::Arc;

use raw_pipeline::histogram::Histogram;
use serde::Serialize;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct PreviewMeta {
    pub asset_id: Uuid,
    pub width: u32,
    pub height: u32,
    pub renderer: String,
    pub histogram: Histogram,
}

#[derive(Clone, Default)]
pub struct PreviewMetaStore {
    inner: Arc<RwLock<HashMap<Uuid, PreviewMeta>>>,
}

impl PreviewMetaStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn put(&self, meta: PreviewMeta) -> Uuid {
        let id = Uuid::new_v4();
        self.inner.write().await.insert(id, meta);
        id
    }

    pub async fn get(&self, id: Uuid) -> Option<PreviewMeta> {
        self.inner.read().await.get(&id).cloned()
    }
}
