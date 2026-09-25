use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use lru::LruCache;
use raw_pipeline::finish::decode_watermark;
use raw_pipeline::frame::WatermarkImage;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tokio::fs;
use uuid::Uuid;

use super::blob_store;

const WATERMARK_CACHE_CAP: usize = 4;

#[derive(Debug, thiserror::Error)]
pub enum WatermarkStoreError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("duplicate")]
    Duplicate(WatermarkMeta),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkMeta {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub created_at: String,
}

#[derive(Clone)]
pub struct WatermarkStore {
    pool: SqlitePool,
    dir: PathBuf,
    cache: Arc<Mutex<LruCache<String, Arc<WatermarkImage>>>>,
}

async fn decode(bytes: Vec<u8>) -> Result<WatermarkImage, WatermarkStoreError> {
    tokio::task::spawn_blocking(move || decode_watermark(&bytes))
        .await
        .map_err(|e| WatermarkStoreError::Io(std::io::Error::other(e)))?
        .map_err(|e| WatermarkStoreError::Invalid(e.to_string()))
}

impl WatermarkStore {
    pub fn new(pool: SqlitePool, data_dir: &Path) -> Result<Self, WatermarkStoreError> {
        let dir = data_dir.join("watermarks");
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            pool,
            dir,
            cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(WATERMARK_CACHE_CAP).unwrap(),
            ))),
        })
    }

    fn blob_path(&self, content_hash: &str) -> PathBuf {
        self.dir.join(format!("{content_hash}.png"))
    }

    fn row_to_meta(row: &sqlx::sqlite::SqliteRow) -> WatermarkMeta {
        WatermarkMeta {
            id: row.get("id"),
            name: row.get("name"),
            width: row.get::<i64, _>("width") as u32,
            height: row.get::<i64, _>("height") as u32,
            size: row.get::<i64, _>("size") as u64,
            created_at: row.get("created_at"),
        }
    }

    fn cache_put(&self, content_hash: String, image: Arc<WatermarkImage>) {
        self.cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .put(content_hash, image);
    }

    pub async fn import(
        &self,
        name: &str,
        bytes: &[u8],
    ) -> Result<WatermarkMeta, WatermarkStoreError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(WatermarkStoreError::Invalid("name is empty".into()));
        }
        let image = decode(bytes.to_vec()).await?;
        let content_hash = blob_store::content_hash(bytes);

        if let Some(existing) = self.find_active_hash(&content_hash).await? {
            return Err(WatermarkStoreError::Duplicate(existing));
        }

        blob_store::write_blob_atomic(&self.blob_path(&content_hash), bytes).await?;
        let width = image.width;
        let height = image.height;
        self.cache_put(content_hash.clone(), Arc::new(image));

        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        if let Err(e) = sqlx::query(
            "INSERT INTO watermarks (id, name, content_hash, size, width, height, deleted, created_at) VALUES (?, ?, ?, ?, ?, ?, 0, ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(&content_hash)
        .bind(bytes.len() as i64)
        .bind(i64::from(width))
        .bind(i64::from(height))
        .bind(&created_at)
        .execute(&self.pool)
        .await
        {
            if blob_store::is_unique_violation(&e)
                && let Some(existing) = self.find_active_hash(&content_hash).await?
            {
                return Err(WatermarkStoreError::Duplicate(existing));
            }
            return Err(e.into());
        }

        Ok(WatermarkMeta {
            id,
            name: name.to_string(),
            width,
            height,
            size: bytes.len() as u64,
            created_at,
        })
    }

    async fn find_active_hash(
        &self,
        content_hash: &str,
    ) -> Result<Option<WatermarkMeta>, WatermarkStoreError> {
        let row = sqlx::query(
            "SELECT id, name, width, height, size, created_at FROM watermarks WHERE content_hash = ? AND deleted = 0",
        )
        .bind(content_hash)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.as_ref().map(Self::row_to_meta))
    }

    pub async fn list(&self) -> Result<Vec<WatermarkMeta>, WatermarkStoreError> {
        let rows = sqlx::query(
            "SELECT id, name, width, height, size, created_at FROM watermarks WHERE deleted = 0 ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(Self::row_to_meta).collect())
    }

    pub async fn soft_delete(&self, id: &str) -> Result<(), WatermarkStoreError> {
        let affected =
            sqlx::query("UPDATE watermarks SET deleted = 1 WHERE id = ? AND deleted = 0")
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();
        if affected == 0 {
            return Err(WatermarkStoreError::NotFound);
        }
        Ok(())
    }

    pub async fn load(&self, id: &str) -> Result<Arc<WatermarkImage>, WatermarkStoreError> {
        let content_hash = self.content_hash(id).await?;
        if let Some(image) = self
            .cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&content_hash)
        {
            return Ok(image.clone());
        }
        let bytes = fs::read(self.blob_path(&content_hash)).await?;
        let image = Arc::new(decode(bytes).await?);
        self.cache_put(content_hash, image.clone());
        Ok(image)
    }

    pub async fn png_bytes(&self, id: &str) -> Result<Vec<u8>, WatermarkStoreError> {
        let content_hash = self.content_hash(id).await?;
        Ok(fs::read(self.blob_path(&content_hash)).await?)
    }

    async fn content_hash(&self, id: &str) -> Result<String, WatermarkStoreError> {
        let row = sqlx::query("SELECT content_hash FROM watermarks WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(WatermarkStoreError::NotFound)?;
        Ok(row.get("content_hash"))
    }
}
