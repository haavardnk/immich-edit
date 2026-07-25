use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use segment::Embedding;
use tokio::fs;
use tokio::sync::Mutex;
use uuid::Uuid;

const MEMORY_SLOTS: usize = 3;
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

#[derive(Clone)]
pub struct EmbeddingCache {
    dir: PathBuf,
    memory: MemorySlots,
}

impl EmbeddingCache {
    pub fn new(cache_dir: &Path) -> Result<Self, EmbeddingCacheError> {
        let dir = cache_dir.join("embeddings");
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            dir,
            memory: Arc::new(Mutex::new(VecDeque::new())),
        })
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
        if let Some(parent) = path.parent()
            && fs::create_dir_all(parent).await.is_ok()
        {
            let _ = fs::write(&path, bytes).await;
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
        let _ = fs::remove_dir_all(dir).await;
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
        let cache = EmbeddingCache::new(dir.path()).unwrap();
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
        let cache = EmbeddingCache::new(dir.path()).unwrap();
        let k = key(Uuid::from_u128(7));
        cache.put(k.clone(), sample()).await;
        cache.purge_owner(k.server_epoch, k.owner).await;
        assert!(cache.get(&k).await.is_none());
    }
}
