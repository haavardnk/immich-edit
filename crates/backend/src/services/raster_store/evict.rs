use std::collections::HashSet;

use lru::LruCache;
use uuid::Uuid;

use super::{RasterStore, RasterStoreError};

pub(super) struct CacheState {
    pub(super) lru: LruCache<(i64, Uuid, String), u64>,
    pub(super) total_bytes: u64,
    pub(super) cap_bytes: u64,
}

impl RasterStore {
    pub fn disk_bytes(&self) -> (u64, u64) {
        let st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        (st.total_bytes, st.cap_bytes)
    }

    async fn pinned_ids(&self) -> Result<HashSet<String>, RasterStoreError> {
        let ids: Vec<String> = sqlx::query_scalar("SELECT DISTINCT raster_id FROM raster_refs")
            .fetch_all(&self.pool)
            .await?;
        Ok(ids.into_iter().collect())
    }

    pub(super) async fn evict_to_cap(&self) -> Result<(), RasterStoreError> {
        {
            let st = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if st.total_bytes <= st.cap_bytes {
                return Ok(());
            }
        }
        let pinned = self.pinned_ids().await?;
        loop {
            let victim: Option<((i64, Uuid, String), u64)> = {
                let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
                if st.total_bytes <= st.cap_bytes {
                    return Ok(());
                }
                let next = st
                    .lru
                    .iter()
                    .rev()
                    .map(|(k, _)| k.clone())
                    .find(|(_, _, id)| !pinned.contains(id));
                match next {
                    Some(key) => {
                        let size = st.lru.pop(&key).unwrap_or(0);
                        st.total_bytes = st.total_bytes.saturating_sub(size);
                        Some((key, size))
                    }
                    None => {
                        let used = st.total_bytes;
                        let cap = st.cap_bytes;
                        drop(st);
                        tracing::warn!(
                            used_bytes = used,
                            cap_bytes = cap,
                            "mask raster cache is over its cap but every remaining raster is \
                             referenced by saved edits; raise MASK_CACHE_MB"
                        );
                        return Ok(());
                    }
                }
            };
            if let Some(((server_epoch, owner, id), _)) = victim {
                let (bin, meta) = self.paths(server_epoch, owner, &id);
                let _ = std::fs::remove_file(&bin);
                let _ = std::fs::remove_file(&meta);
            }
        }
    }

    pub(super) fn touch(&self, server_epoch: i64, owner: Uuid, raster_id: &str) {
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let _ = st.lru.get(&(server_epoch, owner, raster_id.to_string()));
    }

    pub(super) fn insert(&self, server_epoch: i64, owner: Uuid, raster_id: &str, size: u64) {
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(prev) = st
            .lru
            .put((server_epoch, owner, raster_id.to_string()), size)
        {
            st.total_bytes = st.total_bytes.saturating_sub(prev);
        }
        st.total_bytes = st.total_bytes.saturating_add(size);
    }
}
