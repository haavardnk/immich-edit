use std::str::FromStr;

use chrono::Utc;
use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::Edits;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::asset_key::AssetKey;

pub const RENDERER_VERSION: &str = "0.1.0";
const SCHEMA_VERSION: i64 = 2;

mod copies;
mod export_jobs;
mod history;
mod presets;

pub use copies::CopyRecord;

#[derive(Debug, thiserror::Error)]
pub enum EditsStoreError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("migrate: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("corrupt row: {0}")]
    Corrupt(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRecord {
    pub schema_version: u32,
    pub asset_id: AssetKey,
    pub immich_updated_at: Option<String>,
    pub immich_checksum: Option<String>,
    pub renderer_version: String,
    pub manifest: EditManifest,
    pub updated_at: String,
    pub hash: String,
}

impl EditRecord {
    pub fn empty(asset_id: AssetKey) -> Self {
        Self {
            schema_version: SCHEMA_VERSION as u32,
            asset_id,
            immich_updated_at: None,
            immich_checksum: None,
            renderer_version: RENDERER_VERSION.into(),
            manifest: EditManifest::default(),
            updated_at: String::new(),
            hash: Edits::default().stable_hash(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditedAssetEntry {
    pub id: AssetKey,
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
        asset_id: AssetKey,
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
        asset_id: AssetKey,
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
        asset_id: AssetKey,
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
        self.refresh_raster_refs(owner, asset_id).await?;
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
            let Ok(id) = s.parse::<AssetKey>() else {
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
        asset_id: AssetKey,
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
            self.refresh_raster_refs(owner, asset_id).await?;
        }
        Ok(deleted)
    }

    async fn refresh_raster_refs(
        &self,
        owner: Uuid,
        asset_id: AssetKey,
    ) -> Result<(), EditsStoreError> {
        let owner_str = owner.to_string();
        let asset_str = asset_id.to_string();
        let current: Option<String> =
            sqlx::query_scalar("SELECT edits_json FROM edits WHERE user_id = ?1 AND asset_id = ?2")
                .bind(&owner_str)
                .bind(&asset_str)
                .fetch_optional(&self.pool)
                .await?;
        let history: Vec<String> = sqlx::query_scalar(
            "SELECT edits_json FROM edits_history \
             WHERE user_id = ?1 AND asset_id = ?2 AND edits_json IS NOT NULL",
        )
        .bind(&owner_str)
        .bind(&asset_str)
        .fetch_all(&self.pool)
        .await?;
        let mut ids: Vec<String> = Vec::new();
        for json in current.into_iter().chain(history) {
            let Ok(edits) = serde_json::from_str::<Edits>(&json) else {
                continue;
            };
            for id in edits.retained_raster_ids() {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM raster_refs WHERE user_id = ?1 AND asset_id = ?2")
            .bind(&owner_str)
            .bind(&asset_str)
            .execute(&mut *tx)
            .await?;
        for id in &ids {
            sqlx::query(
                "INSERT OR IGNORE INTO raster_refs (user_id, asset_id, raster_id) \
                 VALUES (?1, ?2, ?3)",
            )
            .bind(&owner_str)
            .bind(&asset_str)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn rebuild_raster_refs(&self) -> Result<usize, EditsStoreError> {
        let rows = sqlx::query(
            "SELECT user_id, asset_id, edits_json FROM edits \
             UNION ALL \
             SELECT user_id, asset_id, edits_json FROM edits_history WHERE edits_json IS NOT NULL",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut triples: Vec<(String, String, String)> = Vec::new();
        for row in rows {
            let user_id: String = row.try_get("user_id")?;
            let asset_id: String = row.try_get("asset_id")?;
            let edits_json: String = row.try_get("edits_json")?;
            let Ok(edits) = serde_json::from_str::<Edits>(&edits_json) else {
                continue;
            };
            for id in edits.retained_raster_ids() {
                let triple = (user_id.clone(), asset_id.clone(), id);
                if !triples.contains(&triple) {
                    triples.push(triple);
                }
            }
        }
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM raster_refs")
            .execute(&mut *tx)
            .await?;
        for (user_id, asset_id, raster_id) in &triples {
            sqlx::query(
                "INSERT OR IGNORE INTO raster_refs (user_id, asset_id, raster_id) \
                 VALUES (?1, ?2, ?3)",
            )
            .bind(user_id)
            .bind(asset_id)
            .bind(raster_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(triples.len())
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
        sqlx::query("DELETE FROM raster_refs WHERE user_id = ?1")
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
        sqlx::query("DELETE FROM raster_refs")
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportJobStatus {
    Pending,
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

    fn key() -> AssetKey {
        AssetKey::master(Uuid::new_v4())
    }

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
        if s.get(O, key()).await.unwrap().is_some() {
            panic!("expected none");
        }
    }

    #[tokio::test]
    async fn put_then_get_roundtrips() {
        let s = store().await;
        let id = key();
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
        let id = key();
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
        let id = key();
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
        let id = key();
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
        let id = key();
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
        let id = key();
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
        let id = key();
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
        let asset = key();
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
        if s.get_export_job(O, key(), "x").await.unwrap().is_some() {
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

    #[tokio::test]
    async fn copy_indices_are_monotonic_and_never_reused() {
        let s = store().await;
        let source = uid();
        let first = s.create_copy(O, source, Some("Warm")).await.unwrap();
        let second = s.create_copy(O, source, None).await.unwrap();
        if first.id.to_string() != format!("{source}_1") {
            panic!("first id: {}", first.id);
        }
        if second.id.to_string() != format!("{source}_2") {
            panic!("second id: {}", second.id);
        }
        if first.name.as_deref() != Some("Warm") {
            panic!("name: {:?}", first.name);
        }
        if !s.delete_copy(O, first.id).await.unwrap() {
            panic!("delete returned false");
        }
        let third = s.create_copy(O, source, None).await.unwrap();
        if third.id.to_string() != format!("{source}_3") {
            panic!("third id: {}", third.id);
        }
        let listed: Vec<String> = s
            .list_copies(O, source)
            .await
            .unwrap()
            .iter()
            .map(|c| c.id.to_string())
            .collect();
        if listed != vec![second.id.to_string(), third.id.to_string()] {
            panic!("listed: {listed:?}");
        }
    }

    #[tokio::test]
    async fn delete_copy_cascades_edits_and_history() {
        let s = store().await;
        let source = uid();
        let copy = s.create_copy(O, source, None).await.unwrap();
        s.put(O, copy.id, EditManifest::default(), None, None, None)
            .await
            .unwrap();
        if !s.delete_copy(O, copy.id).await.unwrap() {
            panic!("delete returned false");
        }
        if s.get(O, copy.id).await.unwrap().is_some() {
            panic!("edits survived");
        }
        if !s.list_history(O, copy.id).await.unwrap().is_empty() {
            panic!("history survived");
        }
        if s.list_copies(O, source)
            .await
            .unwrap()
            .iter()
            .any(|c| c.id == copy.id)
        {
            panic!("copy survived");
        }
        if s.delete_copy(O, copy.id).await.unwrap() {
            panic!("second delete reported success");
        }
    }

    #[tokio::test]
    async fn expand_copies_groups_by_source() {
        let s = store().await;
        let a = uid();
        let b = uid();
        let c = uid();
        s.create_copy(O, a, None).await.unwrap();
        s.create_copy(O, a, None).await.unwrap();
        s.create_copy(O, b, None).await.unwrap();
        let map = s.expand_copies(O, &[a, b, c]).await.unwrap();
        if map.get(&a).map(Vec::len) != Some(2) {
            panic!("a: {:?}", map.get(&a));
        }
        if map.get(&b).map(Vec::len) != Some(1) {
            panic!("b: {:?}", map.get(&b));
        }
        if map.contains_key(&c) {
            panic!("c should be absent");
        }
    }

    #[tokio::test]
    async fn rename_copy_returns_none_for_unknown_id() {
        let s = store().await;
        let res = s.rename_copy(O, key(), Some("x")).await.unwrap();
        if res.is_some() {
            panic!("expected none");
        }
    }
}
