use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use raw_pipeline::scopes::{ScopeGrid, ScopeGrids};
use serde::Deserialize;
use tokio::sync::Mutex;
use uuid::Uuid;

const DEFAULT_CAP: usize = 24;
const MAGIC: [u8; 4] = *b"SCOP";
const VERSION: u8 = 1;

pub const HEADER_LEN: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScopeKind {
    Waveform,
    Parade,
    Vectorscope,
}

impl ScopeKind {
    fn tag(self) -> u8 {
        match self {
            Self::Waveform => 0,
            Self::Parade => 1,
            Self::Vectorscope => 2,
        }
    }

    fn select(self, grids: &ScopeGrids) -> &ScopeGrid {
        match self {
            Self::Waveform => &grids.waveform,
            Self::Parade => &grids.parade,
            Self::Vectorscope => &grids.vectorscope,
        }
    }
}

pub fn encode(grids: &ScopeGrids, kind: ScopeKind) -> Vec<u8> {
    let grid = kind.select(grids);
    let mut out = Vec::with_capacity(HEADER_LEN + grid.data.len());
    out.extend_from_slice(&MAGIC);
    out.push(VERSION);
    out.push(kind.tag());
    out.push(grid.channels);
    out.push(0);
    out.extend_from_slice(&grid.width.to_le_bytes());
    out.extend_from_slice(&grid.height.to_le_bytes());
    out.extend_from_slice(&grid.max_count.to_le_bytes());
    out.extend_from_slice(&grid.data);
    out
}

#[derive(Clone)]
pub struct PreviewScopeStore {
    inner: Arc<Mutex<LruCache<Uuid, Arc<ScopeGrids>>>>,
}

impl Default for PreviewScopeStore {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_CAP)
    }
}

impl PreviewScopeStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(cap: usize) -> Self {
        let n = NonZeroUsize::new(cap.max(1)).unwrap();
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(n))),
        }
    }

    pub async fn put(&self, id: Uuid, grids: ScopeGrids) {
        self.inner.lock().await.put(id, Arc::new(grids));
    }

    pub async fn get(&self, id: Uuid) -> Option<Arc<ScopeGrids>> {
        self.inner.lock().await.get(&id).cloned()
    }

    pub async fn clear(&self) {
        self.inner.lock().await.clear();
    }

    #[cfg(test)]
    async fn len(&self) -> usize {
        self.inner.lock().await.len()
    }
}

#[cfg(test)]
mod tests;
