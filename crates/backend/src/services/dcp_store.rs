use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use raw_pipeline::dcp::{DCP_MAX_SOURCE_BYTES, DcpProfile};
use raw_pipeline::parse_dcp;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum DcpStoreError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("duplicate")]
    Duplicate(Box<DcpMeta>),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcpMeta {
    pub id: String,
    pub name: String,
    pub camera_model: Option<String>,
    pub copyright: Option<String>,
    pub bundled: bool,
    pub size: u64,
    pub created_at: String,
}

#[derive(Clone)]
pub struct DcpStore {
    pool: SqlitePool,
    dir: PathBuf,
    cache: Arc<Mutex<HashMap<String, Arc<DcpProfile>>>>,
}

impl DcpStore {
    pub fn new(pool: SqlitePool, cache_dir: &Path) -> Result<Self, DcpStoreError> {
        let dir = cache_dir.join("dcp");
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            pool,
            dir,
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn blob_path(&self, content_hash: &str) -> PathBuf {
        self.dir.join(format!("{content_hash}.dcp"))
    }

    fn row_to_meta(row: &sqlx::sqlite::SqliteRow) -> DcpMeta {
        DcpMeta {
            id: row.get("id"),
            name: row.get("name"),
            camera_model: row.get("camera_model"),
            copyright: row.get("copyright"),
            bundled: row.get::<i64, _>("bundled") != 0,
            size: row.get::<i64, _>("size") as u64,
            created_at: row.get("created_at"),
        }
    }

    pub async fn import(
        &self,
        name: Option<&str>,
        camera: Option<&str>,
        bundled: bool,
        bytes: &[u8],
    ) -> Result<DcpMeta, DcpStoreError> {
        if bytes.len() > DCP_MAX_SOURCE_BYTES {
            return Err(DcpStoreError::Invalid("dcp source too large".into()));
        }
        let profile = parse_dcp(bytes).map_err(|e| DcpStoreError::Invalid(e.to_string()))?;
        let name = name
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| profile.name.clone())
            .unwrap_or_else(|| "DCP profile".to_string());
        let camera_model = camera
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| profile.unique_camera_model.clone());
        let copyright = profile.copyright.clone();
        let content_hash = hex::encode(Sha256::digest(bytes));

        if let Some(existing) = sqlx::query(
            "SELECT id, name, camera_model, copyright, bundled, size, created_at FROM dcp_profiles WHERE content_hash = ? AND deleted = 0",
        )
        .bind(&content_hash)
        .fetch_optional(&self.pool)
        .await?
        {
            return Err(DcpStoreError::Duplicate(Box::new(Self::row_to_meta(
                &existing,
            ))));
        }

        let blob = self.blob_path(&content_hash);
        if !fs::try_exists(&blob).await? {
            fs::write(&blob, bytes).await?;
        }
        self.cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entry(content_hash.clone())
            .or_insert_with(|| Arc::new(profile));

        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        let size = bytes.len() as i64;
        sqlx::query(
            "INSERT INTO dcp_profiles (id, name, camera_model, copyright, content_hash, size, bundled, deleted, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?)",
        )
        .bind(&id)
        .bind(&name)
        .bind(&camera_model)
        .bind(&copyright)
        .bind(&content_hash)
        .bind(size)
        .bind(bundled as i64)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(DcpMeta {
            id,
            name,
            camera_model,
            copyright,
            bundled,
            size: bytes.len() as u64,
            created_at,
        })
    }

    pub async fn import_bundled(&self, dir: &Path) -> Result<usize, DcpStoreError> {
        if !fs::try_exists(dir).await? {
            return Ok(0);
        }
        let mut count = 0;
        let mut rd = fs::read_dir(dir).await?;
        while let Some(entry) = rd.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("dcp") {
                continue;
            }
            let bytes = fs::read(&path).await?;
            match self.import(None, None, true, &bytes).await {
                Ok(_) => count += 1,
                Err(DcpStoreError::Duplicate(_) | DcpStoreError::Invalid(_)) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(count)
    }

    pub async fn list(&self) -> Result<Vec<DcpMeta>, DcpStoreError> {
        let rows = sqlx::query(
            "SELECT id, name, camera_model, copyright, bundled, size, created_at FROM dcp_profiles WHERE deleted = 0 ORDER BY bundled ASC, name ASC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(Self::row_to_meta).collect())
    }

    pub async fn soft_delete(&self, id: &str) -> Result<(), DcpStoreError> {
        let affected =
            sqlx::query("UPDATE dcp_profiles SET deleted = 1 WHERE id = ? AND deleted = 0")
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();
        if affected == 0 {
            return Err(DcpStoreError::NotFound);
        }
        Ok(())
    }

    pub async fn load(&self, id: &str) -> Result<Arc<DcpProfile>, DcpStoreError> {
        let row = sqlx::query("SELECT content_hash FROM dcp_profiles WHERE id = ? AND deleted = 0")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(DcpStoreError::NotFound)?;
        let content_hash: String = row.get("content_hash");
        self.load_hash(&content_hash).await
    }

    pub async fn match_camera(
        &self,
        model: &str,
    ) -> Result<Option<Arc<DcpProfile>>, DcpStoreError> {
        let needle = normalize_model(model);
        if needle.is_empty() {
            return Ok(None);
        }
        let rows = sqlx::query(
            "SELECT content_hash, camera_model FROM dcp_profiles WHERE camera_model IS NOT NULL AND deleted = 0 ORDER BY bundled ASC",
        )
        .fetch_all(&self.pool)
        .await?;
        let hash = rows.iter().find_map(|row| {
            let camera: String = row.get("camera_model");
            let hay = normalize_model(&camera);
            if hay == needle || hay.ends_with(&needle) || needle.ends_with(&hay) {
                Some(row.get::<String, _>("content_hash"))
            } else {
                None
            }
        });
        match hash {
            Some(h) => Ok(Some(self.load_hash(&h).await?)),
            None => Ok(None),
        }
    }

    async fn load_hash(&self, content_hash: &str) -> Result<Arc<DcpProfile>, DcpStoreError> {
        if let Some(p) = self
            .cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(content_hash)
        {
            return Ok(p.clone());
        }
        let bytes = fs::read(self.blob_path(content_hash)).await?;
        let profile = parse_dcp(&bytes).map_err(|e| DcpStoreError::Invalid(e.to_string()))?;
        let profile = Arc::new(profile);
        self.cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(content_hash.to_string(), profile.clone());
        Ok(profile)
    }
}

fn normalize_model(s: &str) -> String {
    s.trim()
        .to_ascii_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::edits_store::EditsStore;

    fn build_dcp(model: Option<&str>) -> Vec<u8> {
        let mut entries: Vec<(u16, u16, u32, Vec<u8>)> = Vec::new();
        let cm: [f32; 9] = [0.6, -0.1, -0.05, -0.3, 1.2, 0.1, 0.02, -0.2, 0.9];
        let mut srational = Vec::new();
        for v in cm {
            srational.extend_from_slice(&((v * 10_000.0).round() as i32).to_le_bytes());
            srational.extend_from_slice(&10_000i32.to_le_bytes());
        }
        entries.push((50721, 10, 9, srational));
        if let Some(m) = model {
            let mut b = m.as_bytes().to_vec();
            b.push(0);
            let count = b.len() as u32;
            entries.push((50708, 2, count, b));
        }
        entries.sort_by_key(|e| e.0);
        let n = entries.len();
        let ifd_off = 8usize;
        let data_off = ifd_off + 2 + n * 12 + 4;
        let mut out = Vec::new();
        out.extend_from_slice(b"II");
        out.extend_from_slice(&42u16.to_le_bytes());
        out.extend_from_slice(&(ifd_off as u32).to_le_bytes());
        out.extend_from_slice(&(n as u16).to_le_bytes());
        let mut blob = Vec::new();
        for (tag, typ, count, bytes) in &entries {
            out.extend_from_slice(&tag.to_le_bytes());
            out.extend_from_slice(&typ.to_le_bytes());
            out.extend_from_slice(&count.to_le_bytes());
            if bytes.len() <= 4 {
                let mut field = bytes.clone();
                field.resize(4, 0);
                out.extend_from_slice(&field);
            } else {
                let off = data_off + blob.len();
                out.extend_from_slice(&(off as u32).to_le_bytes());
                blob.extend_from_slice(bytes);
            }
        }
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&blob);
        out
    }

    async fn store() -> (DcpStore, tempfile::TempDir) {
        let edits = EditsStore::migrated_memory().await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = DcpStore::new(edits.pool(), dir.path()).unwrap();
        (store, dir)
    }

    #[tokio::test]
    async fn import_list_load_delete_roundtrip() {
        let (store, _dir) = store().await;
        let bytes = build_dcp(Some("SONY ILCE-7M4"));
        let meta = store
            .import(Some("My Profile"), None, false, &bytes)
            .await
            .unwrap();
        assert_eq!(meta.name, "My Profile");
        assert_eq!(meta.camera_model.as_deref(), Some("SONY ILCE-7M4"));
        assert!(!meta.bundled);

        let listed = store.list().await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, meta.id);

        store.load(&meta.id).await.unwrap();

        store.soft_delete(&meta.id).await.unwrap();
        assert!(store.list().await.unwrap().is_empty());
        assert!(matches!(
            store.load(&meta.id).await,
            Err(DcpStoreError::NotFound)
        ));
    }

    #[tokio::test]
    async fn import_rejects_duplicate_hash() {
        let (store, _dir) = store().await;
        let bytes = build_dcp(Some("NIKON Z6"));
        store.import(None, None, false, &bytes).await.unwrap();
        let err = store.import(None, None, false, &bytes).await.unwrap_err();
        assert!(matches!(err, DcpStoreError::Duplicate(_)));
    }

    #[tokio::test]
    async fn match_camera_normalizes_model() {
        let (store, _dir) = store().await;
        let bytes = build_dcp(Some("SONY ILCE-7M4"));
        store.import(None, None, false, &bytes).await.unwrap();
        assert!(store.match_camera("ILCE-7M4").await.unwrap().is_some());
        assert!(store.match_camera("Canon EOS R5").await.unwrap().is_none());
    }
}
