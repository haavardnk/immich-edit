use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ml::{Embedding, EmbeddingTensor};
use tokio::fs;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::asset_key::AssetKey;
use crate::safe_path;
use crate::services::cleanup::warn_unless_missing;
use crate::services::disk_lru::DiskLru;

const MEMORY_SLOTS: usize = 3;
const MAGIC: &[u8; 4] = b"IESE";
const FORMAT_VERSION: u32 = 2;
const MAX_TENSORS: usize = 8;

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
    pub asset_id: AssetKey,
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
    disk: Arc<DiskLru>,
}

impl EmbeddingCache {
    pub fn new(cache_dir: &Path, cap_mb: u64) -> Result<Self, EmbeddingCacheError> {
        let dir = cache_dir.join("embeddings");
        let disk = DiskLru::open(&dir, "emb", cap_mb)?;
        Ok(Self {
            dir,
            memory: Arc::new(Mutex::new(VecDeque::new())),
            disk: Arc::new(disk),
        })
    }

    pub async fn disk_bytes(&self) -> (u64, u64) {
        self.disk.bytes().await
    }

    fn path(&self, key: &EmbeddingKey) -> std::io::Result<PathBuf> {
        safe_path::join(
            &self.dir,
            &[
                &key.server_epoch.to_string(),
                &key.owner.to_string(),
                &key.file_name(),
            ],
        )
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
        let path = self.path(key).ok()?;
        let bytes = fs::read(&path).await.ok()?;
        let embedding = decode(&bytes).ok()?;
        self.disk.touch(&path).await;
        let shared = Arc::new(embedding);
        self.push_memory(key.clone(), shared.clone()).await;
        Some(shared)
    }

    pub async fn put(&self, key: EmbeddingKey, embedding: Embedding) -> Arc<Embedding> {
        let shared = Arc::new(embedding);
        self.push_memory(key.clone(), shared.clone()).await;
        let bytes = encode(&shared);
        let size = bytes.len() as u64;
        if let Ok(path) = self.path(&key)
            && let Some(parent) = path.parent()
            && fs::create_dir_all(parent).await.is_ok()
            && fs::write(&path, bytes).await.is_ok()
        {
            self.disk.record(path, size).await;
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
        if let Ok(dir) =
            safe_path::join(&self.dir, &[&server_epoch.to_string(), &owner.to_string()])
        {
            warn_unless_missing(fs::remove_dir_all(&dir).await, &dir);
            self.disk.forget_under(&dir).await;
        }
        let mut mem = self.memory.lock().await;
        mem.retain(|(k, _)| !(k.server_epoch == server_epoch && k.owner == owner));
    }
}

fn encode(embedding: &Embedding) -> Vec<u8> {
    let mut out = Vec::with_capacity(32);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&embedding.scale.to_le_bytes());
    out.extend_from_slice(&embedding.width.to_le_bytes());
    out.extend_from_slice(&embedding.height.to_le_bytes());
    out.extend_from_slice(&(embedding.tensors.len() as u32).to_le_bytes());
    for t in &embedding.tensors {
        out.extend_from_slice(&(t.name.len() as u32).to_le_bytes());
        out.extend_from_slice(t.name.as_bytes());
        out.extend_from_slice(&(t.dims.len() as u32).to_le_bytes());
        for d in &t.dims {
            out.extend_from_slice(&d.to_le_bytes());
        }
        out.extend_from_slice(&(t.values.len() as u64).to_le_bytes());
        for v in &t.values {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    out
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], EmbeddingCacheError> {
        let end = self.at.checked_add(n).ok_or(EmbeddingCacheError::Corrupt)?;
        if end > self.bytes.len() {
            return Err(EmbeddingCacheError::Corrupt);
        }
        let out = &self.bytes[self.at..end];
        self.at = end;
        Ok(out)
    }

    fn u32(&mut self) -> Result<u32, EmbeddingCacheError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn u64(&mut self) -> Result<u64, EmbeddingCacheError> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes(
            b.try_into().map_err(|_| EmbeddingCacheError::Corrupt)?,
        ))
    }

    fn f32(&mut self) -> Result<f32, EmbeddingCacheError> {
        let b = self.take(4)?;
        Ok(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
}

fn decode(bytes: &[u8]) -> Result<Embedding, EmbeddingCacheError> {
    let mut r = Reader { bytes, at: 0 };
    if r.take(4)? != MAGIC || r.u32()? != FORMAT_VERSION {
        return Err(EmbeddingCacheError::Corrupt);
    }
    let scale = r.f32()?;
    let width = r.u32()?;
    let height = r.u32()?;
    let count = r.u32()? as usize;
    if count > MAX_TENSORS {
        return Err(EmbeddingCacheError::Corrupt);
    }
    let mut tensors = Vec::with_capacity(count);
    for _ in 0..count {
        let name_len = r.u32()? as usize;
        let name = std::str::from_utf8(r.take(name_len)?)
            .map_err(|_| EmbeddingCacheError::Corrupt)?
            .to_string();
        let rank = r.u32()? as usize;
        if rank > 8 {
            return Err(EmbeddingCacheError::Corrupt);
        }
        let mut dims = Vec::with_capacity(rank);
        for _ in 0..rank {
            dims.push(r.u64()? as i64);
        }
        let len = r.u64()? as usize;
        let raw = r.take(len.checked_mul(4).ok_or(EmbeddingCacheError::Corrupt)?)?;
        let values = raw
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        tensors.push(EmbeddingTensor { name, dims, values });
    }
    Ok(Embedding {
        tensors,
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
            tensors: vec![
                EmbeddingTensor {
                    name: "image_embed".into(),
                    dims: vec![1, 1, 2, 2],
                    values: vec![0.5, -1.25, 3.0, 0.0],
                },
                EmbeddingTensor {
                    name: "high_res_feats_0".into(),
                    dims: vec![1, 2],
                    values: vec![1.5, 2.5],
                },
            ],
            scale: 0.25,
            width: 4096,
            height: 2048,
        }
    }

    fn key(asset: Uuid) -> EmbeddingKey {
        EmbeddingKey {
            server_epoch: 1,
            owner: Uuid::nil(),
            asset_id: AssetKey::master(asset),
            width: 4096,
            height: 2048,
        }
    }

    #[test]
    fn round_trips_through_bytes() {
        let e = sample();
        let back = decode(&encode(&e)).unwrap();
        assert_eq!(back.tensors, e.tensors);
        assert_eq!(back.scale, e.scale);
        assert_eq!(back.width, e.width);
        assert_eq!(back.height, e.height);
    }

    #[test]
    fn rejects_garbage() {
        assert!(decode(b"nope").is_err());
        assert!(decode(&[]).is_err());
    }

    #[test]
    fn rejects_an_older_format_version() {
        let mut bytes = encode(&sample());
        bytes[4] = 1;
        assert!(decode(&bytes).is_err());
    }

    #[test]
    fn rejects_a_truncated_payload() {
        let bytes = encode(&sample());
        assert!(decode(&bytes[..bytes.len() - 4]).is_err());
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
        assert_eq!(hit.tensors, sample().tensors);
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
    async fn a_disk_hit_survives_the_next_eviction() {
        let dir = tempfile::tempdir().unwrap();
        let cache = EmbeddingCache::new(dir.path(), 1).unwrap();
        let large = || Embedding {
            tensors: vec![EmbeddingTensor {
                name: "image_embed".into(),
                dims: vec![100_000],
                values: vec![0.5; 100_000],
            }],
            ..sample()
        };
        let keys: Vec<EmbeddingKey> = (1..=3u128).map(|i| key(Uuid::from_u128(i))).collect();
        cache.put(keys[0].clone(), large()).await;
        cache.put(keys[1].clone(), large()).await;
        cache.clear_memory().await;
        assert!(cache.get(&keys[0]).await.is_some());
        cache.put(keys[2].clone(), large()).await;
        cache.clear_memory().await;
        assert!(cache.get(&keys[0]).await.is_some());
        assert!(cache.get(&keys[1]).await.is_none());
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
