use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::io::AsyncWriteExt;

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

#[derive(Clone)]
pub struct RasterStore {
    dir: PathBuf,
}

impl RasterStore {
    pub fn new(cache_dir: &Path) -> Result<Self, RasterStoreError> {
        let dir = cache_dir.join("rasters");
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    fn paths(&self, raster_id: &str) -> (PathBuf, PathBuf) {
        let bin = self.dir.join(format!("{raster_id}.r8"));
        let meta = self.dir.join(format!("{raster_id}.json"));
        (bin, meta)
    }

    pub async fn store(
        &self,
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

        let (bin_path, meta_path) = self.paths(&raster_id);
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
        Ok(meta)
    }

    pub async fn load(&self, raster_id: &str) -> Result<(RasterMeta, Vec<u8>), RasterStoreError> {
        let (bin_path, meta_path) = self.paths(raster_id);
        if !fs::try_exists(&meta_path).await? {
            return Err(RasterStoreError::NotFound);
        }
        let meta_bytes = fs::read(&meta_path).await?;
        let meta: RasterMeta = serde_json::from_slice(&meta_bytes)?;
        let bin = fs::read(&bin_path).await?;
        Ok((meta, bin))
    }

    pub async fn meta(&self, raster_id: &str) -> Result<RasterMeta, RasterStoreError> {
        let (_, meta_path) = self.paths(raster_id);
        if !fs::try_exists(&meta_path).await? {
            return Err(RasterStoreError::NotFound);
        }
        let meta_bytes = fs::read(&meta_path).await?;
        let meta: RasterMeta = serde_json::from_slice(&meta_bytes)?;
        Ok(meta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn roundtrip_and_dedupes() {
        let dir = tempdir().unwrap();
        let store = RasterStore::new(dir.path()).unwrap();
        let bytes = vec![0x42u8; 4 * 3];
        let m1 = store.store(&bytes, 4, 3).await.unwrap();
        let m2 = store.store(&bytes, 4, 3).await.unwrap();
        if m1.raster_id != m2.raster_id {
            panic!("same content must dedupe to same id");
        }
        let (meta, loaded) = store.load(&m1.raster_id).await.unwrap();
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
        let store = RasterStore::new(dir.path()).unwrap();
        let r = store.store(&[0u8; 5], 4, 3).await;
        if !matches!(r, Err(RasterStoreError::Invalid(_))) {
            panic!("expected invalid");
        }
    }

    #[tokio::test]
    async fn missing_load_is_not_found() {
        let dir = tempdir().unwrap();
        let store = RasterStore::new(dir.path()).unwrap();
        let r = store.load("deadbeef").await;
        if !matches!(r, Err(RasterStoreError::NotFound)) {
            panic!("expected not found");
        }
    }
}
