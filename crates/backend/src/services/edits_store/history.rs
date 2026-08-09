use chrono::Utc;
use raw_pipeline::edit_manifest::EditManifest;
use raw_pipeline::edits::Edits;
use sqlx::Row;
use uuid::Uuid;

use super::*;

impl EditsStore {
    pub(super) async fn write_history(
        &self,
        owner: Uuid,
        asset_id: AssetKey,
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
        asset_id: AssetKey,
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
        asset_id: AssetKey,
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
        asset_id: AssetKey,
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
        asset_id: AssetKey,
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
}
