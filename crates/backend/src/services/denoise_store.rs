use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use lru::LruCache;
use raw_pipeline::ai_denoise::DenoisePlane;
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::safe_path;
use crate::services::blob_store::write_blob_atomic;
use crate::services::cleanup::warn_unless_missing;
use crate::services::disk_lru::DiskLru;
use crate::services::render::RenderIdentity;

const MEMORY_SLOTS: NonZeroUsize = NonZeroUsize::new(2).unwrap();
const MB: u64 = 1024 * 1024;
const EXTENSION: &str = "iedn";
pub const BAKE_REVISION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlaneKey {
    pub server_epoch: i64,
    pub owner: Uuid,
    pub source: Uuid,
    pub model: String,
}

impl PlaneKey {
    pub fn new(identity: RenderIdentity, source: Uuid, model: String) -> Self {
        Self {
            server_epoch: identity.server_epoch,
            owner: identity.owner,
            source,
            model,
        }
    }

    pub fn plane_id(&self) -> u64 {
        let digest = Sha256::digest(format!(
            "{}/{}/{}/{}/r{BAKE_REVISION}",
            self.server_epoch, self.owner, self.source, self.model
        ));
        let mut id = [0u8; 8];
        id.copy_from_slice(&digest[..8]);
        u64::from_le_bytes(id)
    }

    fn file_name(&self) -> String {
        format!("{}{}", self.source, file_suffix(&self.model))
    }
}

fn file_suffix(model: &str) -> String {
    format!("-{model}-r{BAKE_REVISION}.{EXTENSION}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PurgeTicket {
    all: u64,
    owner: u64,
}

#[derive(Default)]
struct Generations {
    all: u64,
    owners: HashMap<Uuid, u64>,
}

impl Generations {
    fn ticket(&self, owner: Uuid) -> PurgeTicket {
        PurgeTicket {
            all: self.all,
            owner: self.owners.get(&owner).copied().unwrap_or(0),
        }
    }
}

type MemorySlots = Arc<Mutex<LruCache<PlaneKey, Arc<DenoisePlane>>>>;

#[derive(Clone)]
pub struct DenoiseStore {
    dir: PathBuf,
    memory: MemorySlots,
    disk: Arc<DiskLru>,
    generations: Arc<RwLock<Generations>>,
}

impl DenoiseStore {
    pub fn new(cache_dir: &Path, cap_mb: u64) -> std::io::Result<Self> {
        let dir = cache_dir.join("denoise");
        let disk = DiskLru::open(&dir, EXTENSION, cap_mb)?;
        Ok(Self {
            dir,
            memory: Arc::new(Mutex::new(LruCache::new(MEMORY_SLOTS))),
            disk: Arc::new(disk),
            generations: Arc::default(),
        })
    }

    pub async fn disk_bytes(&self) -> (u64, u64) {
        self.disk.bytes().await
    }

    fn path(&self, key: &PlaneKey) -> std::io::Result<PathBuf> {
        safe_path::join(
            &self.dir,
            &[
                &key.server_epoch.to_string(),
                &key.owner.to_string(),
                &key.file_name(),
            ],
        )
    }

    pub async fn contains(&self, key: &PlaneKey) -> bool {
        if self.memory.lock().await.contains(key) {
            return true;
        }
        match self.path(key) {
            Ok(path) => fs::try_exists(&path).await.unwrap_or(false),
            Err(_) => false,
        }
    }

    pub async fn ready_sources(
        &self,
        server_epoch: i64,
        owner: Uuid,
        model: &str,
    ) -> HashSet<Uuid> {
        let mut ready: HashSet<Uuid> = self
            .memory
            .lock()
            .await
            .iter()
            .filter(|(k, _)| k.server_epoch == server_epoch && k.owner == owner && k.model == model)
            .map(|(k, _)| k.source)
            .collect();
        let Ok(dir) = safe_path::join(&self.dir, &[&server_epoch.to_string(), &owner.to_string()])
        else {
            return ready;
        };
        let Ok(mut entries) = fs::read_dir(&dir).await else {
            return ready;
        };
        let suffix = file_suffix(model);
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name();
            if let Some(source) = name
                .to_str()
                .and_then(|name| name.strip_suffix(&suffix))
                .and_then(|source| source.parse::<Uuid>().ok())
            {
                ready.insert(source);
            }
        }
        ready
    }

    pub async fn get(&self, key: &PlaneKey) -> Option<Arc<DenoisePlane>> {
        let path = self.path(key).ok()?;
        let hit = self.memory.lock().await.get(key).cloned();
        if let Some(hit) = hit {
            self.disk.touch(&path).await;
            return Some(hit);
        }
        let ticket = self.ticket(key.owner).await;
        let plane = self.read(key, &path).await?;
        self.remember(key, &path, plane, ticket).await
    }

    pub async fn discard(&self, key: &PlaneKey) {
        self.memory.lock().await.pop(key);
        let Ok(path) = self.path(key) else {
            return;
        };
        tracing::warn!(path = %path.display(), "discarding unusable denoise plane");
        warn_unless_missing(fs::remove_file(&path).await, &path);
        self.disk.forget(&path).await;
    }

    async fn read(&self, key: &PlaneKey, path: &Path) -> Option<DenoisePlane> {
        let bytes = fs::read(path).await.ok()?;
        let expected = key.plane_id();
        let decoded = tokio::task::spawn_blocking(move || raw_pipeline::ai_denoise::decode(&bytes))
            .await
            .ok()?;
        match decoded {
            Ok(plane) if plane.key == expected => Some(plane),
            Ok(_) | Err(_) => {
                self.discard(key).await;
                None
            }
        }
    }

    async fn remember(
        &self,
        key: &PlaneKey,
        path: &Path,
        plane: DenoisePlane,
        ticket: PurgeTicket,
    ) -> Option<Arc<DenoisePlane>> {
        let generations = self.generations.read().await;
        if generations.ticket(key.owner) != ticket {
            return None;
        }
        self.disk.touch(path).await;
        let shared = Arc::new(plane);
        self.memory.lock().await.put(key.clone(), shared.clone());
        drop(generations);
        Some(shared)
    }

    pub async fn ticket(&self, owner: Uuid) -> PurgeTicket {
        self.generations.read().await.ticket(owner)
    }

    pub fn blocking_ticket(&self, owner: Uuid) -> PurgeTicket {
        self.generations.blocking_read().ticket(owner)
    }

    pub async fn put(
        &self,
        key: PlaneKey,
        plane: DenoisePlane,
        ticket: PurgeTicket,
    ) -> std::io::Result<Option<Arc<DenoisePlane>>> {
        let shared = Arc::new(plane);
        let path = self.path(&key)?;
        let encode_from = shared.clone();
        let bytes =
            tokio::task::spawn_blocking(move || raw_pipeline::ai_denoise::encode(&encode_from))
                .await
                .map_err(std::io::Error::other)?
                .map_err(std::io::Error::other)?;
        let size = bytes.len() as u64;
        let (_, cap) = self.disk.bytes().await;
        if size > cap {
            return Err(std::io::Error::other(format!(
                "the denoised photo needs {} MB, more than DENOISE_CACHE_MB ({} MB)",
                size.div_ceil(MB),
                cap / MB
            )));
        }
        let generations = self.generations.read().await;
        if generations.ticket(key.owner) != ticket {
            return Ok(None);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        write_blob_atomic(&path, &bytes).await?;
        self.disk.record(path, size).await;
        self.memory.lock().await.put(key, shared.clone());
        drop(generations);
        Ok(Some(shared))
    }

    pub async fn purge_owner(&self, owner: Uuid) {
        let mut generations = self.generations.write().await;
        *generations.owners.entry(owner).or_default() += 1;
        let mut memory = self.memory.lock().await;
        let stale: Vec<PlaneKey> = memory
            .iter()
            .filter(|(k, _)| k.owner == owner)
            .map(|(k, _)| k.clone())
            .collect();
        for k in stale {
            memory.pop(&k);
        }
        drop(memory);
        let mut epochs = match fs::read_dir(&self.dir).await {
            Ok(entries) => entries,
            Err(error) => {
                warn_unless_missing(Err(error), &self.dir);
                return;
            }
        };
        let owner_dir = owner.to_string();
        while let Ok(Some(epoch)) = epochs.next_entry().await {
            let Ok(dir) = safe_path::join(&epoch.path(), &[&owner_dir]) else {
                continue;
            };
            warn_unless_missing(fs::remove_dir_all(&dir).await, &dir);
            self.disk.forget_under(&dir).await;
        }
        drop(generations);
    }

    pub async fn purge_all(&self) {
        let mut generations = self.generations.write().await;
        generations.all += 1;
        self.memory.lock().await.clear();
        self.disk.forget_under(&self.dir).await;
        warn_unless_missing(fs::remove_dir_all(&self.dir).await, &self.dir);
        warn_unless_missing(fs::create_dir_all(&self.dir).await, &self.dir);
        drop(generations);
    }
}

#[cfg(test)]
mod tests;
