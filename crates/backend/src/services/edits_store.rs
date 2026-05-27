use std::str::FromStr;

use chrono::Utc;
use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::Edits;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

pub const RENDERER_VERSION: &str = "0.1.0";
const SCHEMA_VERSION: i64 = 2;

#[derive(Debug, thiserror::Error)]
pub enum EditsStoreError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("migrate: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRecord {
    pub schema_version: u32,
    pub asset_id: Uuid,
    pub immich_updated_at: Option<String>,
    pub immich_checksum: Option<String>,
    pub renderer_version: String,
    pub manifest: EditManifest,
    pub updated_at: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditedAssetEntry {
    pub id: Uuid,
    pub hash: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditHistoryEntry {
    pub id: i64,
    pub manifest_hash: String,
    pub deleted: bool,
    pub edits: Option<Edits>,
    pub created_at: String,
}

const HISTORY_LIMIT_PER_ASSET: i64 = 50;

#[derive(Debug, Clone)]
pub struct EditsStore {
    pool: SqlitePool,
}

impl EditsStore {
    pub async fn connect(database_url: &str) -> Result<Self, EditsStoreError> {
        let opts = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(opts)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn migrated_memory() -> Result<Self, EditsStoreError> {
        let opts = SqliteConnectOptions::from_str("sqlite::memory:")?.create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn ready(&self) -> Result<(), EditsStoreError> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    pub async fn migration_version(&self) -> Result<Option<i64>, EditsStoreError> {
        let row = sqlx::query("SELECT MAX(version) AS v FROM _sqlx_migrations")
            .fetch_optional(&self.pool)
            .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        Ok(row.try_get::<Option<i64>, _>("v")?)
    }

    pub async fn get(&self, asset_id: Uuid) -> Result<Option<EditRecord>, EditsStoreError> {
        let row = sqlx::query(
            "SELECT edits_json, schema_version, renderer_version, immich_updated_at, \
             immich_checksum, updated_at FROM edits WHERE asset_id = ?1",
        )
        .bind(asset_id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let edits_json: String = row.try_get("edits_json")?;
        let schema_version: i64 = row.try_get("schema_version")?;
        let renderer_version: String = row.try_get("renderer_version")?;
        let immich_updated_at: Option<String> = row.try_get("immich_updated_at")?;
        let immich_checksum: Option<String> = row.try_get("immich_checksum")?;
        let updated_at: String = row.try_get("updated_at")?;
        let edits: Edits = serde_json::from_str(&edits_json)?;
        let hash = edits.stable_hash();
        Ok(Some(EditRecord {
            schema_version: schema_version as u32,
            asset_id,
            immich_updated_at,
            immich_checksum,
            renderer_version,
            manifest: EditManifest::from_edits(&edits),
            updated_at,
            hash,
        }))
    }

    pub async fn get_edits_or_default(&self, asset_id: Uuid) -> Result<Edits, EditsStoreError> {
        let row = sqlx::query("SELECT edits_json FROM edits WHERE asset_id = ?1")
            .bind(asset_id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        let Some(row) = row else {
            return Ok(Edits::default());
        };
        let edits_json: String = row.try_get("edits_json")?;
        let edits: Edits = serde_json::from_str(&edits_json)?;
        Ok(edits)
    }

    pub async fn put(
        &self,
        asset_id: Uuid,
        manifest: EditManifest,
        immich_updated_at: Option<String>,
        immich_checksum: Option<String>,
    ) -> Result<EditRecord, EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        let edits = manifest.to_edits().clamped();
        let edits_json = serde_json::to_string(&edits)?;
        let renderer_version = RENDERER_VERSION.to_string();
        sqlx::query(
            "INSERT INTO edits (asset_id, edits_json, schema_version, renderer_version, \
             immich_updated_at, immich_checksum, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7) \
             ON CONFLICT(asset_id) DO UPDATE SET \
               edits_json = excluded.edits_json, \
               schema_version = excluded.schema_version, \
               renderer_version = excluded.renderer_version, \
               immich_updated_at = excluded.immich_updated_at, \
               immich_checksum = excluded.immich_checksum, \
               updated_at = excluded.updated_at",
        )
        .bind(asset_id.to_string())
        .bind(&edits_json)
        .bind(SCHEMA_VERSION)
        .bind(&renderer_version)
        .bind(&immich_updated_at)
        .bind(&immich_checksum)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        let hash = edits.stable_hash();
        self.write_history(asset_id, &hash, Some(&edits_json), false)
            .await?;
        Ok(EditRecord {
            schema_version: SCHEMA_VERSION as u32,
            asset_id,
            immich_updated_at,
            immich_checksum,
            renderer_version,
            manifest: EditManifest::from_edits(&edits),
            updated_at: now,
            hash,
        })
    }

    pub async fn list_edited_assets(&self) -> Result<Vec<EditedAssetEntry>, EditsStoreError> {
        let rows = sqlx::query(
            "SELECT asset_id, edits_json, updated_at FROM edits ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let s: String = row.try_get("asset_id")?;
            let Ok(id) = Uuid::parse_str(&s) else {
                continue;
            };
            let edits_json: String = row.try_get("edits_json")?;
            let updated_at: String = row.try_get("updated_at")?;
            let Ok(edits) = serde_json::from_str::<Edits>(&edits_json) else {
                continue;
            };
            out.push(EditedAssetEntry {
                id,
                hash: edits.stable_hash(),
                updated_at,
            });
        }
        Ok(out)
    }

    pub async fn delete(&self, asset_id: Uuid) -> Result<bool, EditsStoreError> {
        let res = sqlx::query("DELETE FROM edits WHERE asset_id = ?1")
            .bind(asset_id.to_string())
            .execute(&self.pool)
            .await?;
        let deleted = res.rows_affected() > 0;
        if deleted {
            let tombstone_hash = Edits::default().stable_hash();
            self.write_history(asset_id, &tombstone_hash, None, true)
                .await?;
        }
        Ok(deleted)
    }

    async fn write_history(
        &self,
        asset_id: Uuid,
        manifest_hash: &str,
        edits_json: Option<&str>,
        deleted: bool,
    ) -> Result<(), EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO edits_history (asset_id, manifest_hash, edits_json, deleted, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(asset_id.to_string())
        .bind(manifest_hash)
        .bind(edits_json)
        .bind(if deleted { 1 } else { 0 })
        .bind(&now)
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "DELETE FROM edits_history WHERE asset_id = ?1 AND id NOT IN (\
                SELECT id FROM edits_history WHERE asset_id = ?1 \
                ORDER BY created_at DESC, id DESC LIMIT ?2\
             )",
        )
        .bind(asset_id.to_string())
        .bind(HISTORY_LIMIT_PER_ASSET)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_history(
        &self,
        asset_id: Uuid,
    ) -> Result<Vec<EditHistoryEntry>, EditsStoreError> {
        let rows = sqlx::query(
            "SELECT id, manifest_hash, edits_json, deleted, created_at \
             FROM edits_history WHERE asset_id = ?1 ORDER BY created_at DESC, id DESC",
        )
        .bind(asset_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let edits_json: Option<String> = row.try_get("edits_json")?;
            let edits = edits_json
                .as_deref()
                .and_then(|s| serde_json::from_str::<Edits>(s).ok());
            let deleted_i: i64 = row.try_get("deleted")?;
            out.push(EditHistoryEntry {
                id: row.try_get("id")?,
                manifest_hash: row.try_get("manifest_hash")?,
                deleted: deleted_i != 0,
                edits,
                created_at: row.try_get("created_at")?,
            });
        }
        Ok(out)
    }

    pub async fn get_history_entry_by_hash(
        &self,
        asset_id: Uuid,
        manifest_hash: &str,
    ) -> Result<Option<EditHistoryEntry>, EditsStoreError> {
        let row = sqlx::query(
            "SELECT id, manifest_hash, edits_json, deleted, created_at \
             FROM edits_history WHERE asset_id = ?1 AND manifest_hash = ?2 \
             ORDER BY created_at DESC, id DESC LIMIT 1",
        )
        .bind(asset_id.to_string())
        .bind(manifest_hash)
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let edits_json: Option<String> = row.try_get("edits_json")?;
        let edits = edits_json
            .as_deref()
            .and_then(|s| serde_json::from_str::<Edits>(s).ok());
        let deleted_i: i64 = row.try_get("deleted")?;
        Ok(Some(EditHistoryEntry {
            id: row.try_get("id")?,
            manifest_hash: row.try_get("manifest_hash")?,
            deleted: deleted_i != 0,
            edits,
            created_at: row.try_get("created_at")?,
        }))
    }

    pub async fn get_export_job(
        &self,
        asset_id: Uuid,
        key: &str,
    ) -> Result<Option<ExportJobRecord>, EditsStoreError> {
        let row = sqlx::query(
            "SELECT request_hash, status, immich_asset_id, filename, upload_status, warnings_json \
             FROM export_jobs WHERE asset_id = ?1 AND idempotency_key = ?2",
        )
        .bind(asset_id.to_string())
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let warnings_json: String = row.try_get("warnings_json")?;
        let warnings: Vec<String> = serde_json::from_str(&warnings_json).unwrap_or_default();
        let immich_str: Option<String> = row.try_get("immich_asset_id")?;
        let immich_asset_id = immich_str.as_deref().and_then(|s| Uuid::parse_str(s).ok());
        let status_str: String = row.try_get("status")?;
        let status = match status_str.as_str() {
            "uploaded" => ExportJobStatus::Uploaded,
            "completed" => ExportJobStatus::Completed,
            _ => ExportJobStatus::Uploaded,
        };
        Ok(Some(ExportJobRecord {
            request_hash: row.try_get("request_hash")?,
            status,
            immich_asset_id,
            filename: row.try_get("filename")?,
            upload_status: row.try_get("upload_status")?,
            warnings,
        }))
    }

    pub async fn put_export_job_uploaded(
        &self,
        asset_id: Uuid,
        key: &str,
        request_hash: &str,
        immich_asset_id: Uuid,
        filename: &str,
        upload_status: &str,
    ) -> Result<(), EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO export_jobs (asset_id, idempotency_key, request_hash, status, \
             immich_asset_id, filename, upload_status, warnings_json, created_at, updated_at) \
             VALUES (?1, ?2, ?3, 'uploaded', ?4, ?5, ?6, '[]', ?7, ?7) \
             ON CONFLICT(asset_id, idempotency_key) DO UPDATE SET \
               status = excluded.status, \
               immich_asset_id = excluded.immich_asset_id, \
               filename = excluded.filename, \
               upload_status = excluded.upload_status, \
               updated_at = excluded.updated_at",
        )
        .bind(asset_id.to_string())
        .bind(key)
        .bind(request_hash)
        .bind(immich_asset_id.to_string())
        .bind(filename)
        .bind(upload_status)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn complete_export_job(
        &self,
        asset_id: Uuid,
        key: &str,
        warnings: &[String],
    ) -> Result<(), EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        let warnings_json = serde_json::to_string(warnings)?;
        sqlx::query(
            "UPDATE export_jobs SET status = 'completed', warnings_json = ?3, updated_at = ?4 \
             WHERE asset_id = ?1 AND idempotency_key = ?2",
        )
        .bind(asset_id.to_string())
        .bind(key)
        .bind(&warnings_json)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportJobStatus {
    Uploaded,
    Completed,
}

#[derive(Debug, Clone)]
pub struct ExportJobRecord {
    pub request_hash: String,
    pub status: ExportJobStatus,
    pub immich_asset_id: Option<Uuid>,
    pub filename: Option<String>,
    pub upload_status: Option<String>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uid() -> Uuid {
        Uuid::new_v4()
    }

    async fn store() -> EditsStore {
        EditsStore::migrated_memory().await.unwrap()
    }

    fn manifest_with(edits: Edits) -> EditManifest {
        EditManifest::from_edits(&edits)
    }

    #[tokio::test]
    async fn get_missing_returns_none() {
        let s = store().await;
        if s.get(uid()).await.unwrap().is_some() {
            panic!("expected none");
        }
    }

    #[tokio::test]
    async fn put_then_get_roundtrips() {
        let s = store().await;
        let id = uid();
        let manifest = manifest_with(Edits {
            basic: raw_pipeline::edits::BasicEdits {
                exposure_ev: 1.0,
                ..Default::default()
            },
            geometry: raw_pipeline::edits::GeometryEdits {
                rotate: 90,
                ..Default::default()
            },
            ..Default::default()
        });
        let saved = s
            .put(
                id,
                manifest,
                Some("2026-01-01T00:00:00Z".into()),
                Some("abc".into()),
            )
            .await
            .unwrap();
        if saved.asset_id != id {
            panic!("id");
        }
        let loaded = s.get(id).await.unwrap().unwrap();
        let edits = loaded.manifest.to_edits();
        if edits.basic.exposure_ev != 1.0 || edits.geometry.rotate != 90 {
            panic!("edits");
        }
        if loaded.immich_checksum.as_deref() != Some("abc") {
            panic!("checksum");
        }
    }

    #[tokio::test]
    async fn put_clamps_invalid_values() {
        let s = store().await;
        let id = uid();
        let manifest = manifest_with(Edits {
            basic: raw_pipeline::edits::BasicEdits {
                exposure_ev: 99.0,
                ..Default::default()
            },
            geometry: raw_pipeline::edits::GeometryEdits {
                rotate: 33,
                ..Default::default()
            },
            ..Default::default()
        });
        let saved = s.put(id, manifest, None, None).await.unwrap();
        let edits = saved.manifest.to_edits();
        if edits.basic.exposure_ev > 5.0 {
            panic!("not clamped: {}", edits.basic.exposure_ev);
        }
        if edits.geometry.rotate != 0 {
            panic!("rotate not snapped: {}", edits.geometry.rotate);
        }
    }

    #[tokio::test]
    async fn delete_removes() {
        let s = store().await;
        let id = uid();
        s.put(id, EditManifest::default(), None, None)
            .await
            .unwrap();
        if !s.delete(id).await.unwrap() {
            panic!("first delete");
        }
        if s.delete(id).await.unwrap() {
            panic!("second delete should be false");
        }
        if s.get(id).await.unwrap().is_some() {
            panic!("still present");
        }
    }

    #[tokio::test]
    async fn put_overwrites() {
        let s = store().await;
        let id = uid();
        s.put(
            id,
            manifest_with(Edits {
                basic: raw_pipeline::edits::BasicEdits {
                    exposure_ev: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            }),
            None,
            None,
        )
        .await
        .unwrap();
        s.put(
            id,
            manifest_with(Edits {
                basic: raw_pipeline::edits::BasicEdits {
                    exposure_ev: 2.0,
                    ..Default::default()
                },
                ..Default::default()
            }),
            None,
            None,
        )
        .await
        .unwrap();
        let loaded = s.get(id).await.unwrap().unwrap();
        if loaded.manifest.to_edits().basic.exposure_ev != 2.0 {
            panic!("overwrite");
        }
    }

    #[tokio::test]
    async fn migration_version_reports_latest() {
        let s = store().await;
        let v = s.migration_version().await.unwrap();
        if v.is_none() {
            panic!("missing");
        }
    }

    #[tokio::test]
    async fn export_job_roundtrip_and_complete() {
        let s = store().await;
        let asset = uid();
        let new_id = uid();
        s.put_export_job_uploaded(asset, "k1", "h1", new_id, "f.jpg", "created")
            .await
            .unwrap();
        let r = s.get_export_job(asset, "k1").await.unwrap().unwrap();
        if r.status != ExportJobStatus::Uploaded
            || r.immich_asset_id != Some(new_id)
            || r.request_hash != "h1"
        {
            panic!("uploaded mismatch: {r:?}");
        }
        s.complete_export_job(asset, "k1", &["w1".into()])
            .await
            .unwrap();
        let r = s.get_export_job(asset, "k1").await.unwrap().unwrap();
        if r.status != ExportJobStatus::Completed || r.warnings != vec!["w1".to_string()] {
            panic!("completed mismatch: {r:?}");
        }
    }

    #[tokio::test]
    async fn export_job_missing_returns_none() {
        let s = store().await;
        if s.get_export_job(uid(), "x").await.unwrap().is_some() {
            panic!("expected none");
        }
    }
}
