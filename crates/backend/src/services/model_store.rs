use std::path::{Path, PathBuf};

use chrono::Utc;
use segment::CatalogEntry;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tokio::fs;
use uuid::Uuid;

use crate::services::blob_store;

pub const MAX_MODEL_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ModelStoreError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("checksum mismatch for {id}: expected {expected}, got {actual}")]
    Checksum {
        id: String,
        expected: String,
        actual: String,
    },
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMeta {
    pub id: String,
    pub catalog_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub content_hash: String,
    pub size: u64,
    pub license: String,
    pub source_url: Option<String>,
    pub created_at: String,
}

#[derive(Clone)]
pub struct ModelStore {
    pool: SqlitePool,
    dir: PathBuf,
}

impl ModelStore {
    pub fn new(pool: SqlitePool, cache_dir: &Path) -> Result<Self, ModelStoreError> {
        let dir = cache_dir.join("models");
        std::fs::create_dir_all(&dir)?;
        Ok(Self { pool, dir })
    }

    pub fn blob_path(&self, content_hash: &str) -> PathBuf {
        self.dir.join(format!("{content_hash}.onnx"))
    }

    fn row_to_meta(row: &sqlx::sqlite::SqliteRow) -> ModelMeta {
        ModelMeta {
            id: row.get("id"),
            catalog_id: row.get("catalog_id"),
            name: row.get("name"),
            kind: row.get("kind"),
            content_hash: row.get("content_hash"),
            size: row.get::<i64, _>("size") as u64,
            license: row.get("license"),
            source_url: row.get("source_url"),
            created_at: row.get("created_at"),
        }
    }

    pub async fn install_verified(
        &self,
        entry: &CatalogEntry,
        bytes: &[u8],
    ) -> Result<ModelMeta, ModelStoreError> {
        if bytes.is_empty() {
            return Err(ModelStoreError::Invalid("empty model file".into()));
        }
        if bytes.len() as u64 > MAX_MODEL_BYTES {
            return Err(ModelStoreError::Invalid("model exceeds size limit".into()));
        }
        let content_hash = blob_store::content_hash(bytes);
        if content_hash != entry.sha256 {
            return Err(ModelStoreError::Checksum {
                id: entry.id.to_string(),
                expected: entry.sha256.to_string(),
                actual: content_hash,
            });
        }

        blob_store::write_blob_atomic(&self.blob_path(&content_hash), bytes).await?;

        if let Some(existing) = self.find_by_catalog(entry.id).await? {
            return Ok(existing);
        }

        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        let size = bytes.len() as i64;
        let result = sqlx::query(
            "INSERT INTO models (id, catalog_id, name, kind, content_hash, size, license, source_url, deleted, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, ?)",
        )
        .bind(&id)
        .bind(entry.id)
        .bind(entry.name)
        .bind(entry.kind.as_str())
        .bind(&content_hash)
        .bind(size)
        .bind(entry.license)
        .bind(entry.url)
        .bind(&created_at)
        .execute(&self.pool)
        .await;

        if let Err(e) = result {
            if blob_store::is_unique_violation(&e) {
                return self
                    .find_by_catalog(entry.id)
                    .await?
                    .ok_or(ModelStoreError::NotFound);
            }
            return Err(ModelStoreError::Db(e));
        }

        Ok(ModelMeta {
            id,
            catalog_id: Some(entry.id.to_string()),
            name: entry.name.to_string(),
            kind: entry.kind.as_str().to_string(),
            content_hash,
            size: bytes.len() as u64,
            license: entry.license.to_string(),
            source_url: Some(entry.url.to_string()),
            created_at,
        })
    }

    pub async fn find_by_catalog(
        &self,
        catalog_id: &str,
    ) -> Result<Option<ModelMeta>, ModelStoreError> {
        let row = sqlx::query(
            "SELECT id, catalog_id, name, kind, content_hash, size, license, source_url, created_at FROM models WHERE catalog_id = ? AND deleted = 0",
        )
        .bind(catalog_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.as_ref().map(Self::row_to_meta))
    }

    pub async fn resolve_path(&self, catalog_id: &str) -> Result<PathBuf, ModelStoreError> {
        let meta = self
            .find_by_catalog(catalog_id)
            .await?
            .ok_or(ModelStoreError::NotFound)?;
        let path = self.blob_path(&meta.content_hash);
        if !fs::try_exists(&path).await? {
            return Err(ModelStoreError::NotFound);
        }
        Ok(path)
    }

    pub async fn remove(&self, catalog_id: &str) -> Result<(), ModelStoreError> {
        let meta = self
            .find_by_catalog(catalog_id)
            .await?
            .ok_or(ModelStoreError::NotFound)?;
        sqlx::query("UPDATE models SET deleted = 1 WHERE id = ?")
            .bind(&meta.id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM model_prefs WHERE catalog_id = ?")
            .bind(catalog_id)
            .execute(&self.pool)
            .await?;
        let _ = fs::remove_file(self.blob_path(&meta.content_hash)).await;
        Ok(())
    }

    pub async fn preferred(&self, kind: &str) -> Result<Option<String>, ModelStoreError> {
        let row = sqlx::query("SELECT catalog_id FROM model_prefs WHERE kind = ?")
            .bind(kind)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.get("catalog_id")))
    }

    pub async fn set_preferred(&self, kind: &str, catalog_id: &str) -> Result<(), ModelStoreError> {
        if self.find_by_catalog(catalog_id).await?.is_none() {
            return Err(ModelStoreError::NotFound);
        }
        sqlx::query(
            "INSERT INTO model_prefs (kind, catalog_id, updated_at) VALUES (?, ?, ?) \
             ON CONFLICT(kind) DO UPDATE SET catalog_id = excluded.catalog_id, updated_at = excluded.updated_at",
        )
        .bind(kind)
        .bind(catalog_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::edits_store::EditsStore;
    use segment::catalog;

    async fn store() -> (ModelStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let edits = EditsStore::migrated_memory().await.unwrap();
        let store = ModelStore::new(edits.pool(), dir.path()).unwrap();
        (store, dir)
    }

    fn entry() -> CatalogEntry {
        let mut e = catalog::find("ormbg").unwrap().clone();
        e.sha256 = "";
        e
    }

    #[tokio::test]
    async fn rejects_checksum_mismatch() {
        let (store, _dir) = store().await;
        let entry = catalog::find("ormbg").unwrap();
        let err = store
            .install_verified(entry, b"not the real model")
            .await
            .unwrap_err();
        assert!(matches!(err, ModelStoreError::Checksum { .. }));
    }

    #[tokio::test]
    async fn rejects_empty_model() {
        let (store, _dir) = store().await;
        let entry = catalog::find("ormbg").unwrap();
        let err = store.install_verified(entry, b"").await.unwrap_err();
        assert!(matches!(err, ModelStoreError::Invalid(_)));
    }

    #[tokio::test]
    async fn preference_requires_install_and_clears_on_remove() {
        let (store, _dir) = store().await;
        assert!(matches!(
            store.set_preferred("subject", "ormbg").await.unwrap_err(),
            ModelStoreError::NotFound
        ));

        let bytes = b"pretend onnx payload".to_vec();
        let mut e = entry();
        let hash = blob_store::content_hash(&bytes);
        e.sha256 = Box::leak(hash.into_boxed_str());
        store.install_verified(&e, &bytes).await.unwrap();

        store.set_preferred("subject", "ormbg").await.unwrap();
        assert_eq!(
            store.preferred("subject").await.unwrap().as_deref(),
            Some("ormbg")
        );

        store.remove("ormbg").await.unwrap();
        assert!(store.preferred("subject").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn install_is_idempotent_and_resolvable() {
        let (store, _dir) = store().await;
        let bytes = b"pretend onnx payload".to_vec();
        let mut e = entry();
        let hash = blob_store::content_hash(&bytes);
        e.sha256 = Box::leak(hash.into_boxed_str());

        let first = store.install_verified(&e, &bytes).await.unwrap();
        let second = store.install_verified(&e, &bytes).await.unwrap();
        assert_eq!(first.id, second.id);
        assert!(store.find_by_catalog("ormbg").await.unwrap().is_some());

        let path = store.resolve_path("ormbg").await.unwrap();
        assert_eq!(tokio::fs::read(&path).await.unwrap(), bytes);

        store.remove("ormbg").await.unwrap();
        assert!(store.find_by_catalog("ormbg").await.unwrap().is_none());
        assert!(matches!(
            store.resolve_path("ormbg").await.unwrap_err(),
            ModelStoreError::NotFound
        ));
    }
}
