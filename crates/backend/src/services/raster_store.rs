use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub const MAX_RASTER_EDGE: u32 = 8192;
pub const MAX_RASTER_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum RasterStoreError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RasterMeta {
    pub raster_id: String,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub created_at: String,
}

struct CacheState {
    lru: LruCache<(i64, Uuid, String), u64>,
    total_bytes: u64,
    cap_bytes: u64,
}

#[derive(Clone)]
pub struct RasterStore {
    dir: PathBuf,
    state: Arc<Mutex<CacheState>>,
}

impl RasterStore {
    pub fn new(cache_dir: &Path, cap_mb: u64) -> Result<Self, RasterStoreError> {
        let dir = cache_dir.join("rasters");
        std::fs::create_dir_all(&dir)?;
        let cap_bytes = cap_mb.saturating_mul(1024 * 1024);
        let mut entries: Vec<((i64, Uuid, String), u64, std::time::SystemTime)> = Vec::new();
        for epoch_entry in std::fs::read_dir(&dir)? {
            let epoch_entry = epoch_entry?;
            if !epoch_entry.file_type()?.is_dir() {
                continue;
            }
            let Some(epoch) = epoch_entry
                .file_name()
                .to_str()
                .and_then(|value| value.parse().ok())
            else {
                continue;
            };
            for owner_entry in std::fs::read_dir(epoch_entry.path())? {
                let owner_entry = owner_entry?;
                if !owner_entry.file_type()?.is_dir() {
                    continue;
                }
                let Some(owner) = owner_entry
                    .file_name()
                    .to_str()
                    .and_then(|value| Uuid::parse_str(value).ok())
                else {
                    continue;
                };
                for entry in std::fs::read_dir(owner_entry.path())? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) != Some("r8") {
                        continue;
                    }
                    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                        continue;
                    };
                    let metadata = entry.metadata()?;
                    let modified = metadata
                        .modified()
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    entries.push(((epoch, owner, stem.to_string()), metadata.len(), modified));
                }
            }
        }
        entries.sort_by_key(|(_, _, t)| *t);
        let mut lru: LruCache<(i64, Uuid, String), u64> = LruCache::unbounded();
        let mut total: u64 = 0;
        for (key, size, _) in entries {
            total = total.saturating_add(size);
            lru.put(key, size);
        }
        let state = Arc::new(Mutex::new(CacheState {
            lru,
            total_bytes: total,
            cap_bytes,
        }));
        let store = Self { dir, state };
        store.evict_to_cap();
        Ok(store)
    }

    fn paths(&self, server_epoch: i64, owner: Uuid, raster_id: &str) -> (PathBuf, PathBuf) {
        let owner_dir = self
            .dir
            .join(server_epoch.to_string())
            .join(owner.to_string());
        let bin = owner_dir.join(format!("{raster_id}.r8"));
        let meta = owner_dir.join(format!("{raster_id}.json"));
        (bin, meta)
    }

    fn evict_to_cap(&self) {
        loop {
            let victim: Option<((i64, Uuid, String), u64)> = {
                let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
                if st.total_bytes <= st.cap_bytes || st.lru.is_empty() {
                    return;
                }
                let (k, v) = st.lru.pop_lru().expect("non-empty");
                st.total_bytes = st.total_bytes.saturating_sub(v);
                Some((k, v))
            };
            if let Some(((server_epoch, owner, id), _)) = victim {
                let (bin, meta) = self.paths(server_epoch, owner, &id);
                let _ = std::fs::remove_file(&bin);
                let _ = std::fs::remove_file(&meta);
            }
        }
    }

    fn touch(&self, server_epoch: i64, owner: Uuid, raster_id: &str) {
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let _ = st.lru.get(&(server_epoch, owner, raster_id.to_string()));
    }

    fn insert(&self, server_epoch: i64, owner: Uuid, raster_id: &str, size: u64) {
        {
            let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(prev) = st
                .lru
                .put((server_epoch, owner, raster_id.to_string()), size)
            {
                st.total_bytes = st.total_bytes.saturating_sub(prev);
            }
            st.total_bytes = st.total_bytes.saturating_add(size);
        }
        self.evict_to_cap();
    }

    pub async fn store(
        &self,
        server_epoch: i64,
        owner: Uuid,
        bytes: &[u8],
        width: u32,
        height: u32,
    ) -> Result<RasterMeta, RasterStoreError> {
        if width == 0 || height == 0 || width > MAX_RASTER_EDGE || height > MAX_RASTER_EDGE {
            return Err(RasterStoreError::Invalid(format!(
                "dims {width}x{height} out of range"
            )));
        }
        let expected = width as usize * height as usize;
        if bytes.len() != expected {
            return Err(RasterStoreError::Invalid(format!(
                "byte length {} != width*height {expected}",
                bytes.len()
            )));
        }
        if bytes.len() > MAX_RASTER_BYTES {
            return Err(RasterStoreError::Invalid(format!(
                "raster size {} exceeds cap",
                bytes.len()
            )));
        }

        let mut hasher = Sha256::new();
        hasher.update(width.to_le_bytes());
        hasher.update(height.to_le_bytes());
        hasher.update(bytes);
        let raster_id = hex::encode(hasher.finalize());

        let (bin_path, meta_path) = self.paths(server_epoch, owner, &raster_id);
        if let Some(owner_dir) = bin_path.parent() {
            fs::create_dir_all(owner_dir).await?;
        }
        if !fs::try_exists(&bin_path).await? {
            let mut f = fs::File::create(&bin_path).await?;
            f.write_all(bytes).await?;
            f.sync_all().await?;
        }
        let meta = RasterMeta {
            raster_id: raster_id.clone(),
            width,
            height,
            size: bytes.len() as u64,
            created_at: Utc::now().to_rfc3339(),
        };
        if !fs::try_exists(&meta_path).await? {
            let json = serde_json::to_vec_pretty(&meta)?;
            fs::write(&meta_path, json).await?;
        }
        self.insert(server_epoch, owner, &raster_id, bytes.len() as u64);
        Ok(meta)
    }

    pub async fn load(
        &self,
        server_epoch: i64,
        owner: Uuid,
        raster_id: &str,
    ) -> Result<(RasterMeta, Vec<u8>), RasterStoreError> {
        let (bin_path, meta_path) = self.paths(server_epoch, owner, raster_id);
        if !fs::try_exists(&meta_path).await? {
            return Err(RasterStoreError::NotFound);
        }
        let meta_bytes = fs::read(&meta_path).await?;
        let meta: RasterMeta = serde_json::from_slice(&meta_bytes)?;
        let bin = fs::read(&bin_path).await?;
        self.touch(server_epoch, owner, raster_id);
        Ok((meta, bin))
    }

    pub async fn meta(
        &self,
        server_epoch: i64,
        owner: Uuid,
        raster_id: &str,
    ) -> Result<RasterMeta, RasterStoreError> {
        let (_, meta_path) = self.paths(server_epoch, owner, raster_id);
        if !fs::try_exists(&meta_path).await? {
            return Err(RasterStoreError::NotFound);
        }
        let meta_bytes = fs::read(&meta_path).await?;
        let meta: RasterMeta = serde_json::from_slice(&meta_bytes)?;
        self.touch(server_epoch, owner, raster_id);
        Ok(meta)
    }

    pub async fn purge_owner(&self, owner: Uuid) -> Result<(), RasterStoreError> {
        let mut epochs = fs::read_dir(&self.dir).await?;
        while let Some(epoch) = epochs.next_entry().await? {
            let path = epoch.path().join(owner.to_string());
            if let Err(error) = fs::remove_dir_all(path).await
                && error.kind() != std::io::ErrorKind::NotFound
            {
                return Err(error.into());
            }
        }
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let keys: Vec<_> = state
            .lru
            .iter()
            .filter(|((_, key_owner, _), _)| *key_owner == owner)
            .map(|(key, _)| key.clone())
            .collect();
        for key in keys {
            if let Some(size) = state.lru.pop(&key) {
                state.total_bytes = state.total_bytes.saturating_sub(size);
            }
        }
        Ok(())
    }

    pub async fn purge_all(&self) -> Result<(), RasterStoreError> {
        if let Err(error) = fs::remove_dir_all(&self.dir).await
            && error.kind() != std::io::ErrorKind::NotFound
        {
            return Err(error.into());
        }
        fs::create_dir_all(&self.dir).await?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.lru.clear();
        state.total_bytes = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn unlimited() -> u64 {
        1024
    }

    fn owner() -> Uuid {
        Uuid::nil()
    }

    fn epoch() -> i64 {
        1
    }

    #[tokio::test]
    async fn roundtrip_and_dedupes() {
        let dir = tempdir().unwrap();
        let store = RasterStore::new(dir.path(), unlimited()).unwrap();
        let bytes = vec![0x42u8; 4 * 3];
        let m1 = store.store(epoch(), owner(), &bytes, 4, 3).await.unwrap();
        let m2 = store.store(epoch(), owner(), &bytes, 4, 3).await.unwrap();
        if m1.raster_id != m2.raster_id {
            panic!("same content must dedupe to same id");
        }
        let (meta, loaded) = store.load(epoch(), owner(), &m1.raster_id).await.unwrap();
        if loaded != bytes {
            panic!("roundtrip mismatch");
        }
        if meta.width != 4 || meta.height != 3 {
            panic!("dims mismatch");
        }
    }

    #[tokio::test]
    async fn rejects_size_mismatch() {
        let dir = tempdir().unwrap();
        let store = RasterStore::new(dir.path(), unlimited()).unwrap();
        let r = store.store(epoch(), owner(), &[0u8; 5], 4, 3).await;
        if !matches!(r, Err(RasterStoreError::Invalid(_))) {
            panic!("expected invalid");
        }
    }

    #[tokio::test]
    async fn missing_load_is_not_found() {
        let dir = tempdir().unwrap();
        let store = RasterStore::new(dir.path(), unlimited()).unwrap();
        let r = store.load(epoch(), owner(), "deadbeef").await;
        if !matches!(r, Err(RasterStoreError::NotFound)) {
            panic!("expected not found");
        }
    }

    #[tokio::test]
    async fn evicts_oldest_when_over_cap() {
        let dir = tempdir().unwrap();
        let store = {
            let bytes = (1024 * 1024) as u64;
            let cap_mb = (3 * bytes).div_ceil(1024 * 1024);
            RasterStore::new(dir.path(), cap_mb).unwrap()
        };
        let mk = |fill: u8| vec![fill; 1024 * 1024];
        let a = store
            .store(epoch(), owner(), &mk(1), 1024, 1024)
            .await
            .unwrap();
        let b = store
            .store(epoch(), owner(), &mk(2), 1024, 1024)
            .await
            .unwrap();
        let c = store
            .store(epoch(), owner(), &mk(3), 1024, 1024)
            .await
            .unwrap();
        let _ = store.load(epoch(), owner(), &a.raster_id).await.unwrap();
        let d = store
            .store(epoch(), owner(), &mk(4), 1024, 1024)
            .await
            .unwrap();
        if store.load(epoch(), owner(), &b.raster_id).await.is_ok() {
            panic!("b should have been evicted (least recently used)");
        }
        for id in [&a.raster_id, &c.raster_id, &d.raster_id] {
            if store.load(epoch(), owner(), id).await.is_err() {
                panic!("{id} should survive");
            }
        }
    }

    #[tokio::test]
    async fn populates_lru_from_existing_dir() {
        let dir = tempdir().unwrap();
        {
            let store = RasterStore::new(dir.path(), 1024).unwrap();
            for fill in 1u8..=3 {
                store
                    .store(epoch(), owner(), &vec![fill; 1024 * 1024], 1024, 1024)
                    .await
                    .unwrap();
            }
        }
        let reopened = RasterStore::new(dir.path(), 2).unwrap();
        let state = reopened.state.lock().unwrap();
        if state.total_bytes > state.cap_bytes {
            panic!(
                "reopen should evict to cap: total={} cap={}",
                state.total_bytes, state.cap_bytes
            );
        }
    }
}
