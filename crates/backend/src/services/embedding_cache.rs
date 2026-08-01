use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use lru::LruCache;
use segment::Embedding;
use tokio::fs;
use tokio::sync::Mutex;
use uuid::Uuid;

const MEMORY_SLOTS: usize = 3;
const DISK_SLOTS: usize = 8192;
const MB: u64 = 1024 * 1024;
const MAGIC: &[u8; 4] = b"IESE";

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingCacheError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("corrupt embedding cache entry")]
    Corrupt,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmbeddingKey {
    pub server_epoch: i64,
    pub owner: Uuid,
    pub asset_id: Uuid,
    pub width: u32,
    pub height: u32,
}

impl EmbeddingKey {
    fn file_name(&self) -> String {
        format!("{}-{}x{}.emb", self.asset_id, self.width, self.height)
    }
}

type MemorySlots = Arc<Mutex<VecDeque<(EmbeddingKey, Arc<Embedding>)>>>;

struct DiskState {
    lru: LruCache<PathBuf, u64>,
    total_bytes: u64,
    cap_bytes: u64,
}

#[derive(Clone)]
pub struct EmbeddingCache {
    dir: PathBuf,
    memory: MemorySlots,
    disk: Arc<Mutex<DiskState>>,
}

fn collect_files(dir: &Path, out: &mut Vec<(PathBuf, u64, SystemTime)>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let meta = entry.metadata()?;
        if meta.is_dir() {
            collect_files(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("emb") {
            out.push((path, meta.len(), meta.modified()?));
        }
    }
    Ok(())
}

impl EmbeddingCache {
    pub fn new(cache_dir: &Path, cap_mb: u64) -> Result<Self, EmbeddingCacheError> {
        let dir = cache_dir.join("embeddings");
        std::fs::create_dir_all(&dir)?;

        let mut entries: Vec<(PathBuf, u64, SystemTime)> = Vec::new();
        collect_files(&dir, &mut entries)?;
        entries.sort_by_key(|(_, _, modified)| *modified);

        let mut lru = LruCache::new(NonZeroUsize::new(DISK_SLOTS).unwrap());
        let mut total_bytes = 0u64;
        for (path, size, _) in entries {
            total_bytes = total_bytes.saturating_add(size);
            lru.put(path, size);
        }

        let mut state = DiskState {
            lru,
            total_bytes,
            cap_bytes: cap_mb.saturating_mul(MB),
        };
        while state.total_bytes > state.cap_bytes {
            let Some((path, size)) = state.lru.pop_lru() else {
                break;
            };
            state.total_bytes = state.total_bytes.saturating_sub(size);
            let _ = std::fs::remove_file(&path);
        }

        Ok(Self {
            dir,
            memory: Arc::new(Mutex::new(VecDeque::new())),
            disk: Arc::new(Mutex::new(state)),
        })
    }

    pub async fn disk_bytes(&self) -> (u64, u64) {
        let st = self.disk.lock().await;
        (st.total_bytes, st.cap_bytes)
    }

    async fn evict_to_cap(&self) {
        loop {
            let victim = {
                let mut st = self.disk.lock().await;
                if st.total_bytes <= st.cap_bytes {
                    return;
                }
                match st.lru.pop_lru() {
                    Some((path, size)) => {
                        st.total_bytes = st.total_bytes.saturating_sub(size);
                        path
                    }
                    None => return,
                }
            };
            let _ = fs::remove_file(&victim).await;
        }
    }

    fn path(&self, key: &EmbeddingKey) -> PathBuf {
        self.dir
            .join(key.server_epoch.to_string())
            .join(key.owner.to_string())
            .join(key.file_name())
    }

    pub async fn get(&self, key: &EmbeddingKey) -> Option<Arc<Embedding>> {
        {
            let mut mem = self.memory.lock().await;
            if let Some(pos) = mem.iter().position(|(k, _)| k == key) {
                let hit = mem.remove(pos).expect("position is valid");
                mem.push_front(hit.clone());
                return Some(hit.1);
            }
        }
        let bytes = fs::read(self.path(key)).await.ok()?;
        let embedding = decode(&bytes).ok()?;
        let shared = Arc::new(embedding);
        self.push_memory(key.clone(), shared.clone()).await;
        Some(shared)
    }

    pub async fn put(&self, key: EmbeddingKey, embedding: Embedding) -> Arc<Embedding> {
        let shared = Arc::new(embedding);
        self.push_memory(key.clone(), shared.clone()).await;
        let path = self.path(&key);
        let bytes = encode(&shared);
        let size = bytes.len() as u64;
        if let Some(parent) = path.parent()
            && fs::create_dir_all(parent).await.is_ok()
            && fs::write(&path, bytes).await.is_ok()
        {
            {
                let mut st = self.disk.lock().await;
                if let Some(previous) = st.lru.pop(&path) {
                    st.total_bytes = st.total_bytes.saturating_sub(previous);
                }
                st.total_bytes = st.total_bytes.saturating_add(size);
                st.lru.put(path, size);
            }
            self.evict_to_cap().await;
        }
        shared
    }

    async fn push_memory(&self, key: EmbeddingKey, embedding: Arc<Embedding>) {
        let mut mem = self.memory.lock().await;
        mem.retain(|(k, _)| k != &key);
        mem.push_front((key, embedding));
        while mem.len() > MEMORY_SLOTS {
            mem.pop_back();
        }
    }

    pub async fn clear_memory(&self) {
        self.memory.lock().await.clear();
    }

    pub async fn purge_owner(&self, server_epoch: i64, owner: Uuid) {
        let dir = self
            .dir
            .join(server_epoch.to_string())
            .join(owner.to_string());
        let _ = fs::remove_dir_all(&dir).await;
        {
            let mut st = self.disk.lock().await;
            let stale: Vec<PathBuf> = st
                .lru
                .iter()
                .map(|(path, _)| path.clone())
                .filter(|path| path.starts_with(&dir))
                .collect();
            for path in stale {
                if let Some(size) = st.lru.pop(&path) {
                    st.total_bytes = st.total_bytes.saturating_sub(size);
                }
            }
        }
        let mut mem = self.memory.lock().await;
        mem.retain(|(k, _)| !(k.server_epoch == server_epoch && k.owner == owner));
    }
}

fn encode(embedding: &Embedding) -> Vec<u8> {
    let mut out = Vec::with_capacity(32 + embedding.values.len() * 4);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(embedding.dims.len() as u32).to_le_bytes());
    for d in &embedding.dims {
        out.extend_from_slice(&d.to_le_bytes());
    }
    out.extend_from_slice(&embedding.scale.to_le_bytes());
    out.extend_from_slice(&embedding.width.to_le_bytes());
    out.extend_from_slice(&embedding.height.to_le_bytes());
    for v in &embedding.values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

fn decode(bytes: &[u8]) -> Result<Embedding, EmbeddingCacheError> {
    if bytes.len() < 8 || &bytes[..4] != MAGIC {
        return Err(EmbeddingCacheError::Corrupt);
    }
    let rank = u32::from_le_bytes(
        bytes[4..8]
            .try_into()
            .map_err(|_| EmbeddingCacheError::Corrupt)?,
    ) as usize;
    let dims_end = 8 + rank * 8;
    let header_end = dims_end + 12;
    if rank > 8 || bytes.len() < header_end {
        return Err(EmbeddingCacheError::Corrupt);
    }
    let mut dims = Vec::with_capacity(rank);
    for i in 0..rank {
        let start = 8 + i * 8;
        dims.push(i64::from_le_bytes(
            bytes[start..start + 8]
                .try_into()
                .map_err(|_| EmbeddingCacheError::Corrupt)?,
        ));
    }
    let scale = f32::from_le_bytes(
        bytes[dims_end..dims_end + 4]
            .try_into()
            .map_err(|_| EmbeddingCacheError::Corrupt)?,
    );
    let width = u32::from_le_bytes(
        bytes[dims_end + 4..dims_end + 8]
            .try_into()
            .map_err(|_| EmbeddingCacheError::Corrupt)?,
    );
    let height = u32::from_le_bytes(
        bytes[dims_end + 8..header_end]
            .try_into()
            .map_err(|_| EmbeddingCacheError::Corrupt)?,
    );
    let payload = &bytes[header_end..];
    if payload.len() % 4 != 0 {
        return Err(EmbeddingCacheError::Corrupt);
    }
    let values = payload
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    Ok(Embedding {
        values,
        dims,
        scale,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Embedding {
        Embedding {
            values: vec![0.5, -1.25, 3.0, 0.0],
            dims: vec![1, 1, 2, 2],
            scale: 0.25,
            width: 4096,
            height: 2048,
        }
    }

    fn key(asset: Uuid) -> EmbeddingKey {
        EmbeddingKey {
            server_epoch: 1,
            owner: Uuid::nil(),
            asset_id: asset,
            width: 4096,
            height: 2048,
        }
    }

    #[test]
    fn round_trips_through_bytes() {
        let e = sample();
        let back = decode(&encode(&e)).unwrap();
        assert_eq!(back.values, e.values);
        assert_eq!(back.dims, e.dims);
        assert_eq!(back.scale, e.scale);
        assert_eq!(back.width, e.width);
        assert_eq!(back.height, e.height);
    }

    #[test]
    fn rejects_garbage() {
        assert!(decode(b"nope").is_err());
        assert!(decode(&[]).is_err());
    }

    #[tokio::test]
    async fn survives_memory_eviction_via_disk() {
        let dir = tempfile::tempdir().unwrap();
        let cache = EmbeddingCache::new(dir.path(), 512).unwrap();
        let first = key(Uuid::from_u128(1));
        cache.put(first.clone(), sample()).await;
        for i in 2..=(MEMORY_SLOTS as u128 + 2) {
            cache.put(key(Uuid::from_u128(i)), sample()).await;
        }
        assert_eq!(cache.memory.lock().await.len(), MEMORY_SLOTS);
        let hit = cache.get(&first).await.expect("served from disk");
        assert_eq!(hit.values, sample().values);
    }

    #[tokio::test]
    async fn purge_owner_drops_disk_and_memory() {
        let dir = tempfile::tempdir().unwrap();
        let cache = EmbeddingCache::new(dir.path(), 512).unwrap();
        let k = key(Uuid::from_u128(7));
        cache.put(k.clone(), sample()).await;
        cache.purge_owner(k.server_epoch, k.owner).await;
        assert!(cache.get(&k).await.is_none());
    }

    #[tokio::test]
    async fn disk_cache_evicts_over_cap() {
        let dir = tempfile::tempdir().unwrap();
        let cache = EmbeddingCache::new(dir.path(), 1).unwrap();
        let first = key(Uuid::from_u128(1));
        cache.put(first.clone(), sample()).await;
        for i in 2..=64u128 {
            cache.put(key(Uuid::from_u128(i)), sample()).await;
        }
        cache.clear_memory().await;
        let (used, cap) = cache.disk_bytes().await;
        if used > cap {
            panic!("embedding disk cache over cap: {used} > {cap}");
        }
    }
}
