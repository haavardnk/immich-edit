use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use raw_pipeline::lut::{LUT_MAX_SOURCE_BYTES, Lut3d};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum LutStoreError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("duplicate")]
    Duplicate(LutMeta),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LutMeta {
    pub id: String,
    pub name: String,
    pub lut_size: u32,
    pub size: u64,
    pub created_at: String,
}

#[derive(Clone)]
pub struct LutStore {
    pool: SqlitePool,
    dir: PathBuf,
    cache: Arc<Mutex<HashMap<String, Arc<Lut3d>>>>,
}

impl LutStore {
    pub fn new(pool: SqlitePool, cache_dir: &Path) -> Result<Self, LutStoreError> {
        let dir = cache_dir.join("luts");
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            pool,
            dir,
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn blob_path(&self, content_hash: &str) -> PathBuf {
        self.dir.join(format!("{content_hash}.cube"))
    }

    fn row_to_meta(row: &sqlx::sqlite::SqliteRow) -> LutMeta {
        LutMeta {
            id: row.get("id"),
            name: row.get("name"),
            lut_size: row.get::<i64, _>("lut_size") as u32,
            size: row.get::<i64, _>("size") as u64,
            created_at: row.get("created_at"),
        }
    }

    pub async fn import(&self, name: &str, bytes: &[u8]) -> Result<LutMeta, LutStoreError> {
        if bytes.len() > LUT_MAX_SOURCE_BYTES {
            return Err(LutStoreError::Invalid("cube source too large".into()));
        }
        let lut = Lut3d::parse_cube(bytes).map_err(|e| LutStoreError::Invalid(e.to_string()))?;
        let name = name.trim();
        if name.is_empty() {
            return Err(LutStoreError::Invalid("name is empty".into()));
        }
        let content_hash = hex::encode(Sha256::digest(bytes));

        if let Some(existing) = sqlx::query(
            "SELECT id, name, lut_size, size, created_at FROM luts WHERE content_hash = ? AND deleted = 0",
        )
        .bind(&content_hash)
        .fetch_optional(&self.pool)
        .await?
        {
            return Err(LutStoreError::Duplicate(Self::row_to_meta(&existing)));
        }

        let blob = self.blob_path(&content_hash);
        if !fs::try_exists(&blob).await? {
            fs::write(&blob, bytes).await?;
        }
        self.cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entry(content_hash.clone())
            .or_insert_with(|| Arc::new(lut.clone()));

        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        let lut_size = lut.size() as i64;
        let size = bytes.len() as i64;
        sqlx::query(
            "INSERT INTO luts (id, name, content_hash, size, lut_size, deleted, created_at) VALUES (?, ?, ?, ?, ?, 0, ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(&content_hash)
        .bind(size)
        .bind(lut_size)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(LutMeta {
            id,
            name: name.to_string(),
            lut_size: lut.size() as u32,
            size: bytes.len() as u64,
            created_at,
        })
    }

    pub async fn list(&self) -> Result<Vec<LutMeta>, LutStoreError> {
        let rows = sqlx::query(
            "SELECT id, name, lut_size, size, created_at FROM luts WHERE deleted = 0 ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(Self::row_to_meta).collect())
    }

    pub async fn soft_delete(&self, id: &str) -> Result<(), LutStoreError> {
        let affected = sqlx::query("UPDATE luts SET deleted = 1 WHERE id = ? AND deleted = 0")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        if affected == 0 {
            return Err(LutStoreError::NotFound);
        }
        Ok(())
    }

    pub async fn load(&self, id: &str) -> Result<Arc<Lut3d>, LutStoreError> {
        let row = sqlx::query("SELECT content_hash FROM luts WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(LutStoreError::NotFound)?;
        let content_hash: String = row.get("content_hash");
        if let Some(lut) = self
            .cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&content_hash)
        {
            return Ok(lut.clone());
        }
        let bytes = fs::read(self.blob_path(&content_hash)).await?;
        let lut = Lut3d::parse_cube(&bytes).map_err(|e| LutStoreError::Invalid(e.to_string()))?;
        let lut = Arc::new(lut);
        self.cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(content_hash, lut.clone());
        Ok(lut)
    }
}
