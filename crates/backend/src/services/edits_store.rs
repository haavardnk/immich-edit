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
    pub action: Option<String>,
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
        let opts = SqliteConnectOptions::from_str("sqlite::memory:")?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> SqlitePool {
        self.pool.clone()
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

    pub async fn get(
        &self,
        owner: Uuid,
        asset_id: Uuid,
    ) -> Result<Option<EditRecord>, EditsStoreError> {
        let row = sqlx::query(
            "SELECT edits_json, schema_version, renderer_version, immich_updated_at, \
             immich_checksum, updated_at FROM edits WHERE user_id = ?2 AND asset_id = ?1",
        )
        .bind(asset_id.to_string())
        .bind(owner.to_string())
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

    pub async fn get_edits_or_default(
        &self,
        owner: Uuid,
        asset_id: Uuid,
    ) -> Result<Edits, EditsStoreError> {
        let row = sqlx::query("SELECT edits_json FROM edits WHERE user_id = ?2 AND asset_id = ?1")
            .bind(asset_id.to_string())
            .bind(owner.to_string())
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
        owner: Uuid,
        asset_id: Uuid,
        manifest: EditManifest,
        immich_updated_at: Option<String>,
        immich_checksum: Option<String>,
        action: Option<&str>,
    ) -> Result<EditRecord, EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        let edits = manifest.to_edits().clamped();
        let edits_json = serde_json::to_string(&edits)?;
        let renderer_version = RENDERER_VERSION.to_string();
        sqlx::query(
            "INSERT INTO edits (user_id, asset_id, edits_json, schema_version, renderer_version, \
             immich_updated_at, immich_checksum, created_at, updated_at) \
             VALUES (?8, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7) \
             ON CONFLICT(user_id, asset_id) DO UPDATE SET \
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
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        let hash = edits.stable_hash();
        self.write_history(owner, asset_id, &hash, Some(&edits_json), false, action)
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

    pub async fn list_edited_assets(
        &self,
        owner: Uuid,
    ) -> Result<Vec<EditedAssetEntry>, EditsStoreError> {
        let rows = sqlx::query(
            "SELECT asset_id, edits_json, updated_at FROM edits WHERE user_id = ?1 \
             ORDER BY updated_at DESC",
        )
        .bind(owner.to_string())
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

    pub async fn delete(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        action: Option<&str>,
    ) -> Result<bool, EditsStoreError> {
        let res = sqlx::query("DELETE FROM edits WHERE user_id = ?2 AND asset_id = ?1")
            .bind(asset_id.to_string())
            .bind(owner.to_string())
            .execute(&self.pool)
            .await?;
        let deleted = res.rows_affected() > 0;
        if deleted {
            let tombstone_hash = Edits::default().stable_hash();
            self.write_history(owner, asset_id, &tombstone_hash, None, true, action)
                .await?;
        }
        Ok(deleted)
    }

    pub async fn purge_owner(&self, owner: Uuid) -> Result<(), EditsStoreError> {
        let owner_str = owner.to_string();
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM edits WHERE user_id = ?1")
            .bind(&owner_str)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM edits_history WHERE user_id = ?1")
            .bind(&owner_str)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM presets WHERE user_id = ?1")
            .bind(&owner_str)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM export_jobs WHERE user_id = ?1")
            .bind(&owner_str)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn purge_all(&self) -> Result<(), EditsStoreError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM edits").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM edits_history")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM presets").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM export_jobs")
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn write_history(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        manifest_hash: &str,
        edits_json: Option<&str>,
        deleted: bool,
        action: Option<&str>,
    ) -> Result<(), EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO edits_history (user_id, asset_id, manifest_hash, edits_json, deleted, created_at, action) \
             VALUES (?7, ?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(asset_id.to_string())
        .bind(manifest_hash)
        .bind(edits_json)
        .bind(if deleted { 1 } else { 0 })
        .bind(&now)
        .bind(action)
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "DELETE FROM edits_history WHERE user_id = ?3 AND asset_id = ?1 AND id NOT IN (\
                SELECT id FROM edits_history WHERE user_id = ?3 AND asset_id = ?1 \
                ORDER BY created_at DESC, id DESC LIMIT ?2\
             )",
        )
        .bind(asset_id.to_string())
        .bind(HISTORY_LIMIT_PER_ASSET)
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_history(
        &self,
        owner: Uuid,
        asset_id: Uuid,
    ) -> Result<Vec<EditHistoryEntry>, EditsStoreError> {
        let rows = sqlx::query(
            "SELECT id, manifest_hash, edits_json, deleted, created_at, action \
             FROM edits_history WHERE user_id = ?2 AND asset_id = ?1 ORDER BY created_at DESC, id DESC",
        )
        .bind(asset_id.to_string())
        .bind(owner.to_string())
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
                action: row.try_get("action")?,
            });
        }
        Ok(out)
    }

    pub async fn restore_to_entry(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        entry: &EditHistoryEntry,
    ) -> Result<Option<EditRecord>, EditsStoreError> {
        let current = self.get(owner, asset_id).await?;
        let immich_updated_at = current.as_ref().and_then(|r| r.immich_updated_at.clone());
        let immich_checksum = current.as_ref().and_then(|r| r.immich_checksum.clone());
        sqlx::query("DELETE FROM edits_history WHERE user_id = ?3 AND asset_id = ?1 AND id > ?2")
            .bind(asset_id.to_string())
            .bind(entry.id)
            .bind(owner.to_string())
            .execute(&self.pool)
            .await?;
        if entry.deleted || entry.edits.is_none() {
            sqlx::query("DELETE FROM edits WHERE user_id = ?2 AND asset_id = ?1")
                .bind(asset_id.to_string())
                .bind(owner.to_string())
                .execute(&self.pool)
                .await?;
            return Ok(None);
        }
        let edits = entry.edits.clone().unwrap().clamped();
        let edits_json = serde_json::to_string(&edits)?;
        let renderer_version = RENDERER_VERSION.to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO edits (user_id, asset_id, edits_json, schema_version, renderer_version, \
             immich_updated_at, immich_checksum, created_at, updated_at) \
             VALUES (?8, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7) \
             ON CONFLICT(user_id, asset_id) DO UPDATE SET \
               edits_json = excluded.edits_json, \
               schema_version = excluded.schema_version, \
               renderer_version = excluded.renderer_version, \
               updated_at = excluded.updated_at",
        )
        .bind(asset_id.to_string())
        .bind(&edits_json)
        .bind(SCHEMA_VERSION)
        .bind(&renderer_version)
        .bind(&immich_updated_at)
        .bind(&immich_checksum)
        .bind(&now)
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        let hash = edits.stable_hash();
        Ok(Some(EditRecord {
            schema_version: SCHEMA_VERSION as u32,
            asset_id,
            immich_updated_at,
            immich_checksum,
            renderer_version,
            manifest: EditManifest::from_edits(&edits),
            updated_at: now,
            hash,
        }))
    }

    pub async fn get_history_entry(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        entry_id: i64,
    ) -> Result<Option<EditHistoryEntry>, EditsStoreError> {
        let row = sqlx::query(
            "SELECT id, manifest_hash, edits_json, deleted, created_at, action \
             FROM edits_history WHERE user_id = ?3 AND asset_id = ?1 AND id = ?2",
        )
        .bind(asset_id.to_string())
        .bind(entry_id)
        .bind(owner.to_string())
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
            action: row.try_get("action")?,
        }))
    }

    pub async fn get_history_entry_by_hash(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        manifest_hash: &str,
    ) -> Result<Option<EditHistoryEntry>, EditsStoreError> {
        let row = sqlx::query(
            "SELECT id, manifest_hash, edits_json, deleted, created_at, action \
             FROM edits_history WHERE user_id = ?3 AND asset_id = ?1 AND manifest_hash = ?2 \
             ORDER BY created_at DESC, id DESC LIMIT 1",
        )
        .bind(asset_id.to_string())
        .bind(manifest_hash)
        .bind(owner.to_string())
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
            action: row.try_get("action")?,
        }))
    }

    pub async fn get_export_job(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        key: &str,
    ) -> Result<Option<ExportJobRecord>, EditsStoreError> {
        let row = sqlx::query(
            "SELECT request_hash, status, immich_asset_id, filename, upload_status, warnings_json \
             FROM export_jobs WHERE user_id = ?3 AND asset_id = ?1 AND idempotency_key = ?2",
        )
        .bind(asset_id.to_string())
        .bind(key)
        .bind(owner.to_string())
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

    #[allow(clippy::too_many_arguments)]
    pub async fn put_export_job_uploaded(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        key: &str,
        request_hash: &str,
        immich_asset_id: Uuid,
        filename: &str,
        upload_status: &str,
    ) -> Result<(), EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO export_jobs (user_id, asset_id, idempotency_key, request_hash, status, \
             immich_asset_id, filename, upload_status, warnings_json, created_at, updated_at) \
             VALUES (?8, ?1, ?2, ?3, 'uploaded', ?4, ?5, ?6, '[]', ?7, ?7) \
             ON CONFLICT(user_id, asset_id, idempotency_key) DO UPDATE SET \
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
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn complete_export_job(
        &self,
        owner: Uuid,
        asset_id: Uuid,
        key: &str,
        warnings: &[String],
    ) -> Result<(), EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        let warnings_json = serde_json::to_string(warnings)?;
        sqlx::query(
            "UPDATE export_jobs SET status = 'completed', warnings_json = ?3, updated_at = ?4 \
             WHERE user_id = ?5 AND asset_id = ?1 AND idempotency_key = ?2",
        )
        .bind(asset_id.to_string())
        .bind(key)
        .bind(&warnings_json)
        .bind(&now)
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_presets(&self, owner: Uuid) -> Result<Vec<PresetRecord>, EditsStoreError> {
        let rows = sqlx::query(
            "SELECT id, name, group_name, manifest_json, created_at, updated_at \
             FROM presets WHERE user_id = ?1 ORDER BY group_name IS NULL, group_name, name",
        )
        .bind(owner.to_string())
        .fetch_all(&self.pool)
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(preset_from_row(&row)?);
        }
        Ok(out)
    }

    pub async fn get_preset(
        &self,
        owner: Uuid,
        id: Uuid,
    ) -> Result<Option<PresetRecord>, EditsStoreError> {
        let row = sqlx::query(
            "SELECT id, name, group_name, manifest_json, created_at, updated_at \
             FROM presets WHERE user_id = ?2 AND id = ?1",
        )
        .bind(id.to_string())
        .bind(owner.to_string())
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        Ok(Some(preset_from_row(&row)?))
    }

    pub async fn create_preset(
        &self,
        owner: Uuid,
        name: &str,
        group_name: Option<&str>,
        manifest: &EditManifest,
    ) -> Result<PresetRecord, EditsStoreError> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        let manifest_json = serde_json::to_string(manifest)?;
        sqlx::query(
            "INSERT INTO presets (id, name, group_name, manifest_json, schema_version, \
             created_at, updated_at, user_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7)",
        )
        .bind(id.to_string())
        .bind(name)
        .bind(group_name)
        .bind(&manifest_json)
        .bind(manifest.schema_version as i64)
        .bind(&now)
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        Ok(PresetRecord {
            id,
            name: name.to_string(),
            group_name: group_name.map(str::to_string),
            manifest: manifest.clone(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update_preset(
        &self,
        owner: Uuid,
        id: Uuid,
        name: &str,
        group_name: Option<&str>,
        manifest: &EditManifest,
    ) -> Result<Option<PresetRecord>, EditsStoreError> {
        let now = Utc::now().to_rfc3339();
        let manifest_json = serde_json::to_string(manifest)?;
        let res = sqlx::query(
            "UPDATE presets SET name = ?2, group_name = ?3, manifest_json = ?4, \
             schema_version = ?5, updated_at = ?6 WHERE user_id = ?7 AND id = ?1",
        )
        .bind(id.to_string())
        .bind(name)
        .bind(group_name)
        .bind(&manifest_json)
        .bind(manifest.schema_version as i64)
        .bind(&now)
        .bind(owner.to_string())
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Ok(None);
        }
        self.get_preset(owner, id).await
    }

    pub async fn delete_preset(&self, owner: Uuid, id: Uuid) -> Result<bool, EditsStoreError> {
        let res = sqlx::query("DELETE FROM presets WHERE user_id = ?2 AND id = ?1")
            .bind(id.to_string())
            .bind(owner.to_string())
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }
}

fn preset_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<PresetRecord, EditsStoreError> {
    let id_str: String = row.try_get("id")?;
    let id = Uuid::parse_str(&id_str).map_err(|_| EditsStoreError::Db(sqlx::Error::RowNotFound))?;
    let manifest_json: String = row.try_get("manifest_json")?;
    let manifest: EditManifest = serde_json::from_str(&manifest_json)?;
    Ok(PresetRecord {
        id,
        name: row.try_get("name")?,
        group_name: row.try_get("group_name")?,
        manifest,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetRecord {
    pub id: Uuid,
    pub name: String,
    pub group_name: Option<String>,
    pub manifest: EditManifest,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const O: Uuid = Uuid::nil();

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
        if s.get(O, uid()).await.unwrap().is_some() {
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
                O,
                id,
                manifest,
                Some("2026-01-01T00:00:00Z".into()),
                Some("abc".into()),
                None,
            )
            .await
            .unwrap();
        if saved.asset_id != id {
            panic!("id");
        }
        let loaded = s.get(O, id).await.unwrap().unwrap();
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
        let saved = s.put(O, id, manifest, None, None, None).await.unwrap();
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
        s.put(O, id, EditManifest::default(), None, None, None)
            .await
            .unwrap();
        if !s.delete(O, id, None).await.unwrap() {
            panic!("first delete");
        }
        if s.delete(O, id, None).await.unwrap() {
            panic!("second delete should be false");
        }
        if s.get(O, id).await.unwrap().is_some() {
            panic!("still present");
        }
    }

    #[tokio::test]
    async fn put_overwrites() {
        let s = store().await;
        let id = uid();
        s.put(
            O,
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
            None,
        )
        .await
        .unwrap();
        s.put(
            O,
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
            None,
        )
        .await
        .unwrap();
        let loaded = s.get(O, id).await.unwrap().unwrap();
        if loaded.manifest.to_edits().basic.exposure_ev != 2.0 {
            panic!("overwrite");
        }
    }

    #[tokio::test]
    async fn put_history_roundtrips_action() {
        let s = store().await;
        let id = uid();
        s.put(
            O,
            id,
            manifest_with(Edits {
                basic: raw_pipeline::edits::BasicEdits {
                    exposure_ev: 0.5,
                    ..Default::default()
                },
                ..Default::default()
            }),
            None,
            None,
            Some("Exposure"),
        )
        .await
        .unwrap();
        let hist = s.list_history(O, id).await.unwrap();
        if hist.len() != 1 || hist[0].action.as_deref() != Some("Exposure") {
            panic!("action not stored: {hist:?}");
        }
        s.put(O, id, EditManifest::default(), None, None, None)
            .await
            .unwrap();
        let hist = s.list_history(O, id).await.unwrap();
        if hist.len() != 2 || hist[0].action.is_some() {
            panic!("missing null-action row: {hist:?}");
        }
    }

    #[tokio::test]
    async fn delete_history_roundtrips_action() {
        let s = store().await;
        let id = uid();
        s.put(O, id, EditManifest::default(), None, None, Some("Auto"))
            .await
            .unwrap();
        s.delete(O, id, Some("Brightness")).await.unwrap();
        let hist = s.list_history(O, id).await.unwrap();
        if !hist[0].deleted || hist[0].action.as_deref() != Some("Brightness") {
            panic!("tombstone action: {hist:?}");
        }
    }

    #[tokio::test]
    async fn history_reads_null_action() {
        let s = store().await;
        let id = uid();
        sqlx::query(
            "INSERT INTO edits_history (asset_id, manifest_hash, edits_json, deleted, created_at) \
             VALUES (?1, ?2, ?3, 0, ?4)",
        )
        .bind(id.to_string())
        .bind("deadbeef")
        .bind("{}")
        .bind("2026-01-01T00:00:00Z")
        .execute(&s.pool)
        .await
        .unwrap();
        let hist = s.list_history(O, id).await.unwrap();
        if hist.len() != 1 || hist[0].action.is_some() {
            panic!("expected null action: {hist:?}");
        }
        let single = s
            .get_history_entry_by_hash(O, id, "deadbeef")
            .await
            .unwrap()
            .unwrap();
        if single.action.is_some() {
            panic!("single null action: {single:?}");
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
        s.put_export_job_uploaded(O, asset, "k1", "h1", new_id, "f.jpg", "created")
            .await
            .unwrap();
        let r = s.get_export_job(O, asset, "k1").await.unwrap().unwrap();
        if r.status != ExportJobStatus::Uploaded
            || r.immich_asset_id != Some(new_id)
            || r.request_hash != "h1"
        {
            panic!("uploaded mismatch: {r:?}");
        }
        s.complete_export_job(O, asset, "k1", &["w1".into()])
            .await
            .unwrap();
        let r = s.get_export_job(O, asset, "k1").await.unwrap().unwrap();
        if r.status != ExportJobStatus::Completed || r.warnings != vec!["w1".to_string()] {
            panic!("completed mismatch: {r:?}");
        }
    }

    #[tokio::test]
    async fn export_job_missing_returns_none() {
        let s = store().await;
        if s.get_export_job(O, uid(), "x").await.unwrap().is_some() {
            panic!("expected none");
        }
    }

    #[tokio::test]
    async fn preset_crud_roundtrips() {
        let s = store().await;
        let manifest = manifest_with(Edits {
            basic: raw_pipeline::edits::BasicEdits {
                exposure_ev: 0.5,
                ..Default::default()
            },
            ..Default::default()
        });
        let created = s
            .create_preset(O, "Warm", Some("Looks"), &manifest)
            .await
            .unwrap();
        if created.name != "Warm" || created.group_name.as_deref() != Some("Looks") {
            panic!("create mismatch: {created:?}");
        }
        let fetched = s.get_preset(O, created.id).await.unwrap().unwrap();
        if fetched.manifest.to_edits().basic.exposure_ev != 0.5 {
            panic!("manifest not persisted");
        }
        let updated = s
            .update_preset(O, created.id, "Cool", None, &EditManifest::default())
            .await
            .unwrap()
            .unwrap();
        if updated.name != "Cool" || updated.group_name.is_some() {
            panic!("update mismatch: {updated:?}");
        }
        let all = s.list_presets(O).await.unwrap();
        if all.len() != 1 {
            panic!("list len: {}", all.len());
        }
        if !s.delete_preset(O, created.id).await.unwrap() {
            panic!("delete returned false");
        }
        if s.get_preset(O, created.id).await.unwrap().is_some() {
            panic!("preset not deleted");
        }
    }

    #[tokio::test]
    async fn update_missing_preset_returns_none() {
        let s = store().await;
        let res = s
            .update_preset(O, uid(), "x", None, &EditManifest::default())
            .await
            .unwrap();
        if res.is_some() {
            panic!("expected none");
        }
    }
}
